// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-285: `CheckedPackageV2::read` over a package whose lock selects a real
//! Semantic IR 2.0.0 domain package (non-empty `types`), through the whole
//! chain the reader runs: the document is admitted and read (FR-322 step 1),
//! each declaration's model declaration node key is recovered (step 2), and
//! a `quire.op.model.dispatch_call` on a model-owned operation is resolved
//! and typed (steps 3 and 4) against the graph.
//!
//! The package and the domain package document are built here, node by node,
//! from FR-322's and QSL FR-092's preimages; nothing of QSpec is copied in.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_package, rebuild_source_map, refresh_identity,
    refusal_cause, sha256_hex,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const STRUCTURAL_NODE: &str = "quire.structural-node/v1";
const APPLICATION_NODE: &str = "quire.application-node/v1";
const IDENTITY: &str = "acme/orders";
const VERSION: &str = "1.0.0";
const ORDER: &str = "ix://acme/orders/Order";

/// QSL FR-092 golden keys of the Integer and Text scalar types.
const T2_INTEGER: &str = "07f6dca966d22bde13d3bb198f12610e57d8e1e04d0476bbab03f405d2b04e32";
const T3_TEXT: &str = "0062ac9eee212061689152b5b0d62c5b3a6ca69ce83c1897b8c0624837ab5659";

fn structural_key(
    tag: &str,
    form: &str,
    semantic_type: Option<&str>,
    owner: Option<Value>,
    body: &Value,
) -> String {
    let mut preimage = json!({
        "version": STRUCTURAL_NODE,
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": semantic_type.map(node_id),
        "declaration": null,
        "recursion": null,
        "body": body,
    });
    if let Some(owner) = owner {
        preimage["owner"] = owner;
    }
    sha256_hex(&canonical(&preimage))
}

fn wire_node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
        "occurrences": [{"role": "type", "ordinal": 0}],
        "body": body,
    })
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(ty), "value_kind": value_kind, "value": value})
}

fn parameter(name: &str, level: &str, ty: &str) -> (String, Value) {
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": literal(T3_TEXT, "text", name)},
        {"term": "binding", "name": "level", "value": literal(T2_INTEGER, "integer", level)},
    ]});
    let key = structural_key("value", "parameter", Some(ty), None, &body);
    let node = wire_node(&key, "value", "parameter", ty, &[], body);
    (key, node)
}

/// One field or operation slot: `typeRef` with the multiplicity `[1, 1]`.
fn slot(identity: Option<&str>, type_ref: &str) -> Value {
    let mut slot = json!({
        "typeRef": type_ref,
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
    });
    if let Some(identity) = identity {
        slot["identity"] = json!(identity);
    }
    slot
}

/// The domain package document: an `Order` with the operation
/// `total(Integer): Integer`, plus `extra` merged into `Order`.
fn domain_document(extra: Value) -> Value {
    let mut order = json!({
        "identity": ORDER, "displayName": "Order",
        "kind": {"module": "acme/orders", "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": [], "fields": [],
        "operations": [{
            "identity": "ix://acme/orders/Order/total",
            "params": [slot(None, "ix://quire/native/Integer")],
            "returns": slot(None, "ix://quire/native/Integer"),
        }],
    });
    if let (Some(order), Some(extra)) = (order.as_object_mut(), extra.as_object()) {
        for (member, value) in extra {
            order.insert(member.clone(), value.clone());
        }
    }
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": IDENTITY, "version": VERSION},
        "constructs": [{
            "kind": {"module": "acme/orders", "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [order],
    })
}

/// A package selecting `document`, holding `Order`'s model declaration node
/// and `receiver.total(argument)` as a `dispatch_call` of the operation
/// named `name`, plus the evidence that supplies the document.
fn package_over(
    document: &Value,
    name: &str,
) -> (Value, quire_contract_ir::CheckedPackageEvidence) {
    let digest = sha256_hex(&canonical(document));
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": IDENTITY, "version": VERSION,
        "digest_domain": "sha256-jcs", "digest": digest,
    }]);

    let integer = structural_key("scalar_type", "integer", None, None, &empty());
    let text = structural_key("scalar_type", "text", None, None, &empty());
    assert_eq!([&integer, &text], [T2_INTEGER, T3_TEXT]);
    let order = structural_key(
        "model",
        "object_type",
        None,
        Some(json!({"kind": "model", "identity": IDENTITY, "version": VERSION, "node": ORDER})),
        &empty(),
    );
    let reference_body = json!({"term": "aggregate", "members": [reference(&order)]});
    let reference_type = structural_key("composite_type", "reference", None, None, &reference_body);
    let (receiver, receiver_node) = parameter("receiver", "0", &reference_type);
    let (argument, argument_node) = parameter("argument", "1", T2_INTEGER);

    let call_body = json!({
        "term": "application",
        "operator": "call",
        "operation": {
            "identity": "quire.op.model.dispatch_call", "laws": [], "mode": null,
            "member": {"kind": "operation", "declaration": node_id(&order), "name": name},
            "leaves": [],
        },
        "result_type": node_id(T2_INTEGER),
        "arguments": [reference(&receiver), reference(&argument)],
    });
    let call = sha256_hex(&canonical(&json!({
        "version": APPLICATION_NODE,
        "node_tag": "expression",
        "semantic_form": "call",
        "semantic_type": node_id(T2_INTEGER),
        "declaration": null,
        "recursion": null,
        "body": call_body,
    })));
    let mut call_dependencies = [receiver.as_str(), argument.as_str(), order.as_str()];
    call_dependencies.sort_unstable();
    let nodes = [
        wire_node(
            T2_INTEGER,
            "scalar_type",
            "integer",
            T2_INTEGER,
            &[],
            empty(),
        ),
        wire_node(T3_TEXT, "scalar_type", "text", T3_TEXT, &[], empty()),
        wire_node(&order, "model", "object_type", &order, &[], empty()),
        wire_node(
            &reference_type,
            "composite_type",
            "reference",
            &reference_type,
            &[order.as_str()],
            reference_body,
        ),
        receiver_node,
        argument_node,
        wire_node(
            &call,
            "expression",
            "call",
            T2_INTEGER,
            &call_dependencies,
            call_body,
        ),
    ];
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let mut evidence = evidence_for(&package);
    evidence.insert_domain_package_document(digest, canonical(document));
    (package, evidence)
}

fn read(
    package: &Value,
    evidence: &quire_contract_ir::CheckedPackageEvidence,
) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn refused(
    package: &Value,
    evidence: &quire_contract_ir::CheckedPackageEvidence,
) -> CheckedPackageRefusal {
    match read(package, evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

/// The position of the `dispatch_call` application in [`package_over`]'s graph.
const CALL: usize = 6;

/// Tracing: TC-048, FR-038-AC-29
#[trace("TC-048", "FR-038-AC-29")]
#[test]
fn tc_048_a_model_owned_operation_admits_through_a_real_semantic_ir_document() {
    let (package, evidence) = package_over(&domain_document(json!({})), "total");
    match read(&package, &evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission, got {other:?}"),
    }
}

/// Tracing: TC-048, FR-038-AC-29
#[trace("TC-048", "FR-038-AC-29")]
#[test]
fn tc_048_a_model_owned_operation_the_document_does_not_declare_refuses() {
    let (package, evidence) = package_over(&domain_document(json!({})), "missing");
    let refusal = refused(&package, &evidence);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some(format!("/semantic_graph/nodes/{CALL}/body/operation/member/name").as_str())
    );
}

/// Tracing: TC-048, FR-038-AC-27, FR-038-AC-28
#[trace("TC-048", "FR-038-AC-27", "FR-038-AC-28")]
#[test]
fn tc_048_a_declaration_refusal_is_located_at_its_model_selection_row() {
    let dangling = domain_document(json!({"supertypes": ["ix://acme/orders/Nope"]}));
    let (package, evidence) = package_over(&dangling, "total");
    assert_eq!(
        refused(&package, &evidence),
        refusal_cause(
            CheckedPackageRefusalCode::MissingDeclaration,
            "/lock/model_selections/0",
            CheckedPackageRefusalCause::MissingName
        )
    );
    // The refusal is the document's, not a stale-digest one: the same
    // document, named by its own digest, was supplied and admitted.
}

/// Tracing: TC-048, FR-038-AC-30
#[trace("TC-048", "FR-038-AC-30")]
#[test]
fn tc_048_reading_a_domain_package_is_charged_to_the_work_limit() {
    let (package, evidence) = package_over(&domain_document(json!({})), "total");
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = 0;
    match CheckedPackageV2::read(&canonical(&package), limits, &evidence) {
        CheckedPackageV2ReadResult::Incomplete(CheckedPackageIncomplete {
            limit_kind,
            limit,
            path,
            ..
        }) => {
            assert_eq!(limit_kind, CheckedPackageLimit::Work);
            assert_eq!(limit, 0);
            assert_eq!(
                path.map(|path| path.to_string()).as_deref(),
                Some("/lock/model_selections/0")
            );
        }
        other => panic!("expected an incomplete read, got {other:?}"),
    }
}
