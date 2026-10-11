// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External pre-read candidate controls; draft-only values need not satisfy the
//! reader grammar. Reader controls separately assemble a real package.

use crate::support::checked_package::{
    canonical, evidence_for, mint_grouped_structural_keys, refresh_identity, rename_node,
    v2_all_families,
};
use ix_trace_rs::trace;
use quire_canonical::{Error, LimitKind, Limits};
use quire_contract_model::{
    derive_application_node_id, ApplicationIdentityContext, ApplicationIdentityError,
    ApplicationNodeDraft, CheckedNodeId, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedSemanticNodeV2,
};
use serde_json::{json, Value};

const BYTES: u64 = 32 * 1024 * 1024;

fn id(value: u8) -> CheckedNodeId {
    CheckedNodeId {
        domain: "quire.checked-semantic-node/v1".into(),
        digest: format!("{value:02x}").repeat(32).into(),
    }
}

fn node(value: u8, body: Value) -> CheckedSemanticNodeV2 {
    CheckedSemanticNodeV2 {
        node_id: id(value),
        schema_version: "quire.checked-semantic-graph/v2".into(),
        node_tag: "function".into(),
        semantic_form: "function".into(),
        semantic_type: id(3),
        dependencies: Vec::new(),
        occurrences: Vec::new(),
        recursion_group: None,
        nominal_identity_preimage: None,
        declaration: None,
        owner: None,
        body,
    }
}

fn application(arguments: Vec<Value>) -> Value {
    json!({"term": "application", "arguments": arguments})
}

fn reference(value: u8) -> Value {
    json!({"term": "reference", "target": id(value)})
}

fn candidate(node: &CheckedSemanticNodeV2) -> CheckedNodeId {
    derive_application_node_id(ApplicationNodeDraft::new(node), None, BYTES)
        .expect("ordinary candidate")
        .candidate_node_id()
        .clone()
}

fn oracle(node: &CheckedSemanticNodeV2, recursion: Value, body: Value) -> Value {
    json!({
        "version": "quire.application-node/v1",
        "node_tag": node.node_tag,
        "semantic_form": node.semantic_form,
        "semantic_type": node.semantic_type,
        "declaration": node.declaration,
        "recursion": recursion,
        "body": body,
    })
}

/// Trace: FR-046-AC-1, FR-046-AC-2
#[trace("FR-046-AC-1", "FR-046-AC-2")]
#[test]
fn ordinary_candidates_bind_the_preimage_fields_before_any_read() {
    let original = node(1, application(vec![reference(9)]));
    let expected = quire_canonical::sha256(
        &oracle(&original, Value::Null, original.body.clone()),
        Limits::new(BYTES),
    )
    .expect("independent preimage")
    .to_string();
    let actual = candidate(&original);
    assert_eq!(actual.domain.as_ref(), "quire.checked-semantic-node/v1");
    assert_eq!(actual.digest.as_ref(), expected);
    let mut selector = original.clone();
    selector.node_id = id(2);
    assert_eq!(candidate(&selector), actual);
    let mut tag = original.clone();
    tag.node_tag = "expression".into();
    let mut form = original.clone();
    form.semantic_form = "call".into();
    let mut ty = original.clone();
    ty.semantic_type = id(4);
    let mut declared = original.clone();
    declared.declaration = Some(serde_json::from_value(json!({"qualified_name": ["f"]})).unwrap());
    let mut renamed = declared.clone();
    renamed.declaration.as_mut().unwrap().qualified_name = vec!["g".into()];
    let mut body = original.clone();
    body.body = application(vec![reference(8)]);
    for changed in [tag, form, ty, declared.clone(), body] {
        assert_ne!(candidate(&changed), actual);
    }
    assert_ne!(candidate(&renamed), candidate(&declared));
}

/// Trace: FR-046-AC-3
#[trace("FR-046-AC-3")]
#[test]
fn grouped_candidates_use_all_members_in_order_and_preserve_external_references() {
    let outside = reference(9);
    let dependency = json!({"term": "dependency_reference", "target": id(1)});
    let body = application(vec![
        reference(2),
        json!({"term": "aggregate", "members": [reference(1)]}),
        json!({"term": "binding", "value": reference(3)}),
        outside.clone(),
        dependency.clone(),
    ]);
    let mut graph = vec![
        node(1, Value::Null),
        node(2, body),
        node(3, application(vec![])),
    ];
    for member in &mut graph {
        member.recursion_group = Some("draft-group".into());
    }
    let context =
        ApplicationIdentityContext::try_new(&graph, ApplicationNodeDraft::new(&graph[1])).unwrap();
    let actual =
        derive_application_node_id(ApplicationNodeDraft::new(&graph[1]), Some(&context), BYTES)
            .unwrap();
    let rewritten = application(vec![
        json!({"term": "group_reference", "ordinal": 1}),
        json!({"term": "aggregate", "members": [{"term": "group_reference", "ordinal": 0}]}),
        json!({"term": "binding", "value": {"term": "group_reference", "ordinal": 2}}),
        outside,
        dependency,
    ]);
    let expected = quire_canonical::sha256(
        &oracle(&graph[1], json!({"size": 3, "ordinal": 1}), rewritten),
        Limits::new(BYTES),
    )
    .unwrap()
    .to_string();
    assert_eq!(actual.candidate_node_id().digest.as_ref(), expected);
    let mut final_graph = graph.clone();
    final_graph[0].node_id = id(4);
    final_graph[1].node_id = actual.candidate_node_id().clone();
    final_graph[2].node_id = id(5);
    final_graph[1].body["arguments"][0]["target"] = json!(final_graph[1].node_id);
    final_graph[1].body["arguments"][1]["members"][0]["target"] = json!(id(4));
    final_graph[1].body["arguments"][2]["value"]["target"] = json!(id(5));
    let final_context = ApplicationIdentityContext::try_new(
        &final_graph,
        ApplicationNodeDraft::new(&final_graph[1]),
    )
    .unwrap();
    let final_candidate = derive_application_node_id(
        ApplicationNodeDraft::new(&final_graph[1]),
        Some(&final_context),
        BYTES,
    )
    .unwrap();
    assert_eq!(actual, final_candidate);
    final_graph[1].body["arguments"][3]["target"] = json!(id(8));
    let changed_context = ApplicationIdentityContext::try_new(
        &final_graph,
        ApplicationNodeDraft::new(&final_graph[1]),
    )
    .unwrap();
    let changed = derive_application_node_id(
        ApplicationNodeDraft::new(&final_graph[1]),
        Some(&changed_context),
        BYTES,
    )
    .unwrap();
    assert_ne!(changed, actual);
}

/// Trace: FR-046-AC-1, FR-046-AC-4
#[trace("FR-046-AC-1", "FR-046-AC-4")]
#[test]
fn draft_context_refusals_have_exact_payloads_and_deterministic_precedence() {
    let ordinary = node(1, application(vec![]));
    let draft = ApplicationNodeDraft::new(&ordinary);
    assert!(matches!(ApplicationIdentityContext::try_new(&[], draft),
        Err(ApplicationIdentityError::MissingTarget { node_id }) if node_id == id(1)));
    let repeated = [
        node(2, Value::Null),
        node(3, Value::Null),
        node(3, Value::Null),
        node(2, Value::Null),
    ];
    assert!(
        matches!(ApplicationIdentityContext::try_new(&repeated, draft),
        Err(ApplicationIdentityError::RepeatedKey { node_id, first_position: 1, repeated_position: 2 }) if node_id == id(3))
    );
    let changed = node(1, application(vec![Value::Bool(true)]));
    assert!(
        matches!(ApplicationIdentityContext::try_new(std::slice::from_ref(&changed), draft),
        Err(ApplicationIdentityError::ContextMismatch { node_id }) if node_id == id(1))
    );
    let context =
        ApplicationIdentityContext::try_new(std::slice::from_ref(&ordinary), draft).unwrap();
    assert_eq!(
        derive_application_node_id(draft, Some(&context), BYTES)
            .unwrap()
            .candidate_node_id(),
        &candidate(&ordinary)
    );
    assert!(
        matches!(derive_application_node_id(ApplicationNodeDraft::new(&changed), Some(&context), 0),
        Err(ApplicationIdentityError::ContextMismatch { node_id }) if node_id == id(1))
    );
    let mut grouped = ordinary.clone();
    grouped.recursion_group = Some("g".into());
    assert!(
        matches!(derive_application_node_id(ApplicationNodeDraft::new(&grouped), None, 0),
        Err(ApplicationIdentityError::MissingContext { node_id }) if node_id == id(1))
    );
    grouped.body = Value::Null;
    assert!(
        matches!(derive_application_node_id(ApplicationNodeDraft::new(&grouped), None, 0),
        Err(ApplicationIdentityError::NonApplication { node_id }) if node_id == id(1))
    );
    // Exhaustive discriminant use is part of the external consumer contract.
    fn typed_kind(error: ApplicationIdentityError) -> u8 {
        match error {
            ApplicationIdentityError::NonApplication { .. } => 0,
            ApplicationIdentityError::MissingContext { .. } => 1,
            ApplicationIdentityError::MissingTarget { .. } => 2,
            ApplicationIdentityError::RepeatedKey { .. } => 3,
            ApplicationIdentityError::ContextMismatch { .. } => 4,
            ApplicationIdentityError::ContextIndexOutOfRange { .. } => 5,
            ApplicationIdentityError::CanonicalEncoding(_) => 6,
        }
    }
    assert_eq!(
        typed_kind(
            derive_application_node_id(ApplicationNodeDraft::new(&grouped), None, BYTES)
                .unwrap_err()
        ),
        0
    );
}

/// Trace: FR-046-AC-5
#[trace("FR-046-AC-5")]
#[test]
fn canonical_byte_boundaries_and_number_refusals_preserve_encoder_errors() {
    let ordinary = node(1, application(vec![reference(9)]));
    let preimage = oracle(&ordinary, Value::Null, ordinary.body.clone());
    let count = u64::try_from(
        quire_canonical::to_vec(&preimage, Limits::new(BYTES))
            .unwrap()
            .len(),
    )
    .unwrap();
    let draft = ApplicationNodeDraft::new(&ordinary);
    for limit in [count, count + 1] {
        assert_eq!(
            derive_application_node_id(draft, None, limit)
                .unwrap()
                .candidate_node_id(),
            &candidate(&ordinary)
        );
    }
    for limit in [0, count - 1] {
        assert!(matches!(derive_application_node_id(draft, None, limit),
            Err(ApplicationIdentityError::CanonicalEncoding(Error::Limit(exceeded)))
                if exceeded.kind == LimitKind::CanonicalBytes && exceeded.bound == limit));
    }
    let wide = node(1, application(vec![json!(9_007_199_254_740_993_u64)]));
    assert!(matches!(
        derive_application_node_id(ApplicationNodeDraft::new(&wide), None, BYTES),
        Err(ApplicationIdentityError::CanonicalEncoding(
            Error::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993)
        ))
    ));
}

fn deep(levels: usize, bottom: bool) -> Value {
    let mut value = Value::Bool(bottom);
    for _ in 0..levels {
        value = Value::Array(vec![value]);
    }
    value
}

fn deep_draft(grouped: bool, bottom: bool, replaced_reference: bool) -> CheckedSemanticNodeV2 {
    let body = if replaced_reference {
        let mut term = reference(1);
        term.as_object_mut()
            .unwrap()
            .insert("extra".into(), deep(100_000, bottom));
        application(vec![term])
    } else {
        application(vec![deep(100_000, bottom)])
    };
    let mut result = node(1, body);
    result.recursion_group = grouped.then(|| "g".into());
    result
}

fn dispose(mut node: CheckedSemanticNodeV2) {
    quire_canonical::drop_value(std::mem::take(&mut node.body));
}

/// Trace: FR-046-AC-4
#[trace("FR-046-AC-4")]
#[test]
fn deep_borrowed_drafts_copy_compare_hash_and_refuse_on_a_small_stack() {
    std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {
        for grouped in [false, true] {
            for replaced_reference in [false, true] {
                let original = deep_draft(grouped, true, replaced_reference);
                let snapshot = deep_draft(grouped, true, replaced_reference);
                let changed = deep_draft(grouped, false, replaced_reference);
                let draft = ApplicationNodeDraft::new(&original);
                let graph = [snapshot];
                let context = ApplicationIdentityContext::try_new(&graph, draft).unwrap();
                let result = derive_application_node_id(draft, Some(&context), BYTES).unwrap();
                assert_eq!(result.candidate_node_id().domain.as_ref(), "quire.checked-semantic-node/v1");
                assert!(matches!(derive_application_node_id(draft, Some(&context), 0),
                    Err(ApplicationIdentityError::CanonicalEncoding(Error::Limit(_)))));
                assert!(matches!(ApplicationIdentityContext::try_new(&graph, ApplicationNodeDraft::new(&changed)),
                    Err(ApplicationIdentityError::ContextMismatch { node_id }) if node_id == id(1)));
                assert!(matches!(derive_application_node_id(ApplicationNodeDraft::new(&changed), Some(&context), BYTES),
                    Err(ApplicationIdentityError::ContextMismatch { node_id }) if node_id == id(1)));
                dispose(original);
                dispose(changed);
                for node in graph { dispose(node); }
            }
        }
    }).unwrap().join().expect("deep candidate controls complete without recursive helper operations");
}

/// Trace: FR-046-AC-6
#[trace("FR-046-AC-6")]
#[test]
fn caller_substitution_admits_and_a_retained_candidate_refuses_a_changed_body() {
    for grouped in [false, true] {
        let mut package = v2_all_families();
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|node| node["body"]["term"] == "application")
            .unwrap();
        if grouped {
            package["semantic_graph"]["nodes"][position]["recursion_group"] = json!("draft");
        }
        let nodes: Vec<CheckedSemanticNodeV2> =
            serde_json::from_value(package["semantic_graph"]["nodes"].clone()).unwrap();
        let draft = ApplicationNodeDraft::new(&nodes[position]);
        let context = ApplicationIdentityContext::try_new(&nodes, draft).unwrap();
        let candidate = derive_application_node_id(draft, Some(&context), BYTES).unwrap();
        let stale = nodes[position].node_id.digest.as_ref();
        rename_node(
            &mut package,
            stale,
            candidate.candidate_node_id().digest.as_ref(),
        );
        if grouped {
            mint_grouped_structural_keys(&mut package);
        }
        refresh_identity(&mut package);
        assert_eq!(
            package["semantic_graph"]["nodes"][position]["node_id"],
            json!(candidate.candidate_node_id())
        );
        assert!(matches!(
            CheckedPackageV2::read(
                &canonical(&package),
                CheckedPackageReadLimits::bounded(),
                &evidence_for(&package)
            ),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        package["semantic_graph"]["nodes"][position]["body"]["operation"]["identity"] =
            json!("changed.operation");
        refresh_identity(&mut package);
        let result = CheckedPackageV2::read(
            &canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&package),
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("changed body must refuse its retained candidate: {result:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::StaleNodeKey)
        );
        assert_ne!(
            refusal.expected_node_id(),
            Some(candidate.candidate_node_id())
        );
        assert!(refusal.expected_node_id().is_some());
    }
}
