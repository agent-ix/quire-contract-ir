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
    canonical, evidence_for, node_id, nominal_package, pointer, rebuild_source_map,
    refresh_identity, refusal_bytes, refusal_cause, sha256_hex,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::sync::LazyLock;

const STRUCTURAL_NODE: &str = "quire.structural-node/v1";
const APPLICATION_NODE: &str = "quire.application-node/v1";
const IDENTITY: &str = "acme/orders";
const VERSION: &str = "1.0.0";
const ORDER: &str = "ix://acme/orders/Order";

/// The self-typed Integer and Text scalar type nodes, keyed by
/// [`structural_key`].
static INTEGER: LazyLock<String> =
    LazyLock::new(|| structural_key("scalar_type", "integer", None, None, &empty()));
static TEXT: LazyLock<String> =
    LazyLock::new(|| structural_key("scalar_type", "text", None, None, &empty()));

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
        {"term": "binding", "name": "name", "value": literal(&TEXT, "text", name)},
        {"term": "binding", "name": "level", "value": literal(&INTEGER, "integer", level)},
    ]});
    let key = structural_key("value", "parameter", Some(ty), None, &body);
    let mut node = wire_node(&key, "value", "parameter", ty, &[], body);
    node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
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
    let package = package_selecting(name, &digest);
    let mut evidence = evidence_for(&package);
    evidence.insert_domain_package_document(digest, canonical(document));
    (package, evidence)
}

/// [`package_over`]'s package, whose one `model_selections` row names `digest`.
fn package_selecting(name: &str, digest: &str) -> Value {
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": IDENTITY,
        "digest_domain": "sha256-jcs", "digest": digest,
    }]);

    let order = structural_key(
        "model",
        "object_type",
        None,
        Some(json!({"kind": "model", "identity": IDENTITY, "node": ORDER})),
        &empty(),
    );
    let reference_body = json!({"term": "aggregate", "members": [reference(&order)]});
    let reference_type = structural_key("composite_type", "reference", None, None, &reference_body);
    let (receiver, receiver_node) = parameter("receiver", "0", &reference_type);
    let (argument, argument_node) = parameter("argument", "1", &INTEGER);

    let call_body = json!({
        "term": "application",
        "operator": "call",
        "operation": {
            "identity": "quire.op.model.dispatch_call", "laws": [], "mode": null,
            "member": {"kind": "operation", "declaration": node_id(&order), "name": name},
            "leaves": [],
        },
        "result_type": node_id(&INTEGER),
        "arguments": [reference(&receiver), reference(&argument)],
    });
    let call = sha256_hex(&canonical(&json!({
        "version": APPLICATION_NODE,
        "node_tag": "expression",
        "semantic_form": "call",
        "semantic_type": node_id(&INTEGER),
        "declaration": null,
        "recursion": null,
        "body": call_body,
    })));
    let mut call_dependencies = [receiver.as_str(), argument.as_str(), order.as_str()];
    call_dependencies.sort_unstable();
    let mut declaring_node = wire_node(&order, "model", "object_type", &order, &[], empty());
    declaring_node["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": ORDER});
    let nodes = [
        wire_node(&INTEGER, "scalar_type", "integer", &INTEGER, &[], empty()),
        wire_node(&TEXT, "scalar_type", "text", &TEXT, &[], empty()),
        declaring_node,
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
            &INTEGER,
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
    package
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

const RELATIONSHIP: &str = "ix://acme/orders/relationship/Order-links-Order";

fn authored_relationship() -> Value {
    json!({
        "identity": RELATIONSHIP, "category": "structural", "composite": false,
        "direction": "bidirectional",
        "sourceEnd": {"type": ORDER, "role": "links", "multiplicity":
            {"lower": 0, "upper": 1, "ordered": false, "unique": true}},
        "targetEnd": {"type": ORDER, "role": "linkedBy", "multiplicity":
            {"lower": 1, "upper": 1, "ordered": false, "unique": true}},
        "origin": {"source": {"sourceIdentity": ORDER, "path": "models/Order.md",
            "startLine": 1, "startColumn": 1}},
    })
}

fn relationship_package(
    relationship: Value,
    role: &str,
) -> (Value, quire_contract_ir::CheckedPackageEvidence) {
    relationship_operation_package(relationship, role, false)
}

fn relationship_operation_package(
    relationship: Value,
    role: &str,
    reaches: bool,
) -> (Value, quire_contract_ir::CheckedPackageEvidence) {
    let document = domain_document(json!({"relationships": [relationship]}));
    let digest = sha256_hex(&canonical(&document));
    let mut package = package_selecting("total", &digest);
    let relationship_key = structural_key(
        "relation",
        "relationship",
        None,
        Some(json!({"kind": "model", "identity": IDENTITY, "node": RELATIONSHIP})),
        &empty(),
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let receiver = nodes[4]["node_id"]["digest"]
        .as_str()
        .expect("receiver")
        .to_owned();
    let reference_type = nodes[3]["node_id"]["digest"]
        .as_str()
        .expect("reference")
        .to_owned();
    let boolean = structural_key("scalar_type", "boolean", None, None, &empty());
    let result_type = if reaches { &boolean } else { &reference_type };
    let form = if reaches { "reachability" } else { "query" };
    let operator = if reaches { "reaches" } else { "query" };
    let operation = if reaches {
        "quire.op.model.reaches"
    } else {
        "quire.op.model.navigate"
    };
    let arguments = if reaches {
        vec![reference(&receiver), reference(&receiver)]
    } else {
        vec![reference(&receiver)]
    };
    let body = json!({
        "term": "application", "operator": operator,
        "operation": {"identity": operation, "laws": [], "mode": null,
            "member": {"kind": "relationship_end", "declaration": node_id(&relationship_key), "name": role},
            "leaves": []},
        "result_type": node_id(result_type),
        "arguments": arguments,
    });
    let application_key = sha256_hex(&canonical(&json!({
        "version": APPLICATION_NODE, "node_tag": "expression", "semantic_form": form,
        "semantic_type": node_id(result_type), "declaration": null,
        "recursion": null, "body": body,
    })));
    let mut dependencies = [receiver.as_str(), relationship_key.as_str()];
    dependencies.sort_unstable();
    nodes[CALL] = wire_node(
        &application_key,
        "expression",
        form,
        result_type,
        &dependencies,
        body,
    );
    let mut relation_node = wire_node(
        &relationship_key,
        "relation",
        "relationship",
        &relationship_key,
        &[],
        empty(),
    );
    relation_node["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": RELATIONSHIP});
    nodes.push(relation_node);
    if reaches {
        nodes.push(wire_node(
            &boolean,
            "scalar_type",
            "boolean",
            &boolean,
            &[],
            empty(),
        ));
    }
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let mut evidence = evidence_for(&package);
    evidence.insert_domain_package_document(digest, canonical(&document));
    (package, evidence)
}

/// Trace: FR-038-AC-165, FR-038-AC-166, FR-038-AC-169, FR-038-AC-170, FR-038-AC-171
#[trace(
    "TC-048",
    "FR-038-AC-165",
    "FR-038-AC-166",
    "FR-038-AC-169",
    "FR-038-AC-170",
    "FR-038-AC-171"
)]
#[test]
fn tc_048_a_selected_fcd_relationship_navigates_through_its_named_end() {
    let (package, evidence) = relationship_package(authored_relationship(), "links");
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let (package, evidence) = relationship_package(authored_relationship(), "missing");
    let refusal = refused(&package, &evidence);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/6/body/operation/member/name")
    );
    assert_eq!(
        refusal.locus.as_ref(),
        Some(
            &serde_json::from_value(package["semantic_graph"]["nodes"][CALL]["node_id"].clone())
                .expect("application id")
        )
    );
    let mut renamed = authored_relationship();
    renamed["sourceEnd"]["role"] = json!("billedTo");
    let (package, evidence) = relationship_package(renamed.clone(), "links");
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingName)
        )
    );
    let (package, evidence) = relationship_package(renamed.clone(), "billedTo");
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    renamed["targetEnd"]["role"] = json!("billedTo");
    let (package, evidence) = relationship_package(renamed, "billedTo");
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            Some(CheckedPackageRefusalCause::AmbiguousName)
        )
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/6/body/operation/member/name")
    );
}

/// Trace: FR-038-AC-169, FR-038-AC-172
#[trace("TC-048", "FR-038-AC-169", "FR-038-AC-172")]
#[test]
fn tc_048_a_selected_self_relationship_reaches_through_either_named_end() {
    for role in ["links", "linkedBy"] {
        let (package, evidence) =
            relationship_operation_package(authored_relationship(), role, true);
        let result = read(&package, &evidence);
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "{role}: {result:?}"
        );
    }
    let mut set_edge = authored_relationship();
    set_edge["targetEnd"]["multiplicity"]["lower"] = json!(2);
    set_edge["targetEnd"]["multiplicity"]["upper"] = json!(3);
    let (package, evidence) = relationship_operation_package(set_edge, "links", true);
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        )
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/6/body/operation/member/name")
    );
}

/// Trace: FR-038-AC-171
#[trace("TC-048", "FR-038-AC-171")]
#[test]
fn tc_048_relationship_navigation_total_work_limit_reports_its_first_unpayable_edge() {
    let (package, evidence) = relationship_package(authored_relationship(), "links");
    let bytes = canonical(&package);
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        CheckedPackageV2::read(&bytes, limits, &evidence)
    };
    let (mut insufficient, mut admitted) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(matches!(
        read_at(admitted),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    while insufficient + 1 < admitted {
        let middle = insufficient + (admitted - insufficient) / 2;
        match read_at(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => admitted = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => insufficient = middle,
            other => panic!("expected work boundary, got {other:?}"),
        }
    }
    assert!(matches!(
        read_at(admitted),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    match read_at(admitted - 1) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, admitted - 1);
            assert_eq!(
                incomplete.path.map(|path| path.to_string()).as_deref(),
                Some("/semantic_graph/nodes/6/body/arguments/0/target")
            );
        }
        other => panic!("expected total-work exhaustion, got {other:?}"),
    }
}

/// Trace: FR-038-AC-165, FR-038-AC-166, FR-038-AC-167, FR-038-AC-168, FR-038-AC-173
#[trace(
    "TC-048",
    "FR-038-AC-165",
    "FR-038-AC-166",
    "FR-038-AC-167",
    "FR-038-AC-168",
    "FR-038-AC-173"
)]
#[test]
fn tc_048_selected_relationship_declaration_refusals_precede_navigation() {
    let mut cases = Vec::new();
    let mut wrong_identity = authored_relationship();
    wrong_identity["identity"] = json!("ix://acme/orders/Order/links");
    cases.push((
        wrong_identity,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::MalformedDeclaration,
    ));
    let mut missing_direction = authored_relationship();
    missing_direction
        .as_object_mut()
        .expect("relationship")
        .remove("direction");
    cases.push((
        missing_direction,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::MalformedDeclaration,
    ));
    let mut no_role = authored_relationship();
    no_role["sourceEnd"]
        .as_object_mut()
        .expect("source end")
        .remove("role");
    cases.push((
        no_role,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::MalformedDeclaration,
    ));
    let mut unresolved = authored_relationship();
    unresolved["targetEnd"]["type"] = json!("ix://acme/orders/Ghost");
    cases.push((
        unresolved,
        CheckedPackageRefusalCode::MissingDeclaration,
        CheckedPackageRefusalCause::MissingName,
    ));
    let mut reversed = authored_relationship();
    reversed["targetEnd"]["multiplicity"]["lower"] = json!(3);
    cases.push((
        reversed,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::UnpreservedModelMeaning,
    ));
    for (relation, code, cause) in cases {
        let (package, evidence) = relationship_package(relation.clone(), "missing");
        let refusal = refused(&package, &evidence);
        assert_eq!(
            (refusal.code, refusal.cause),
            (code, Some(cause)),
            "{relation}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some("/lock/model_selections/0")
        );
    }
    let mut no_inverse = authored_relationship();
    no_inverse["targetEnd"]
        .as_object_mut()
        .expect("target end")
        .remove("role");
    let (package, evidence) = relationship_package(no_inverse, "linkedBy");
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingName)
        )
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/6/body/operation/member/name")
    );
}

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

/// Tracing: TC-048, FR-038-AC-45
#[trace("TC-048", "FR-038-AC-45")]
#[test]
fn tc_048_a_declaration_node_key_is_unchanged_when_the_selected_document_changes() {
    let (selected, selected_evidence) = package_over(&domain_document(json!({})), "total");
    match read(&selected, &selected_evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission, got {other:?}"),
    }

    // A document that differs only in content the keys do not read (its
    // display name), so the selected row's digest changes and its version
    // does not: the keys are unchanged (FR-038-AC-45).
    let renamed = domain_document(json!({"displayName": "Order renamed"}));
    let (digest_only, digest_evidence) = package_over(&renamed, "total");
    assert_ne!(
        selected["lock"]["model_selections"][0]["digest"],
        digest_only["lock"]["model_selections"][0]["digest"]
    );
    match read(&digest_only, &digest_evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission under the changed digest, got {other:?}"),
    }
    assert_eq!(selected["semantic_graph"], digest_only["semantic_graph"]);

    // The same declarations in a document whose only difference is its
    // package version, which the selection no longer names: the keys are
    // version-free (`tc_048_a_selected_document_needs_no_package_version`).
    assert_ne!(
        declaration_key_of(ORDER, IDENTITY),
        declaration_key_of(ORDER, OTHER_PACKAGE),
        "the owner's identity, not its version, distinguishes declaration keys"
    );
}

/// Tracing: TC-048, FR-038-AC-45
#[trace("TC-048", "FR-038-AC-45")]
#[test]
fn tc_048_a_declaration_node_keyed_under_another_domain_package_refuses() {
    // The member's declaration is `Order` keyed under a package the lock does
    // not select: the key resolves to no selected declaration.
    let (package, evidence, _) = reaches_package(
        (ORDER, OTHER_PACKAGE),
        "parent",
        Operand::Reference("unselected"),
        Operand::Reference("unselected"),
    );
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingSelection)
        )
    );
    let unselected = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["owner"]["identity"] == OTHER_PACKAGE)
        .expect("unselected model node");
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some(format!("/semantic_graph/nodes/{unselected}/node_id").as_str())
    );
    // The selected package's own key for the same node resolves.
    let (package, evidence, _) = reaches_package(
        (ORDER, IDENTITY),
        "parent",
        Operand::Reference(ORDER),
        Operand::Reference(ORDER),
    );
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// A selection binds by identity and content digest: the document's own
/// `package.version` is neither required nor read, so a matching document
/// with no version, a non-string one or any string admits, each document
/// selected under its own digest. The model-owned node keys carry no version
/// by construction (the owner is `{kind, identity, node}`, pinned by
/// `tc_048_a_declaration_node_key_is_unchanged_when_the_selected_document_changes`),
/// so admission per version is the oracle here, not graph equality.
///
/// Tracing: TC-048, FR-038-AC-27, FR-038-AC-45, FR-038-AC-64
#[trace("TC-048", "FR-038-AC-27", "FR-038-AC-45", "FR-038-AC-64")]
#[test]
fn tc_048_a_selected_document_needs_no_package_version() {
    let versioned = |version: Option<Value>| {
        let mut document = domain_document(json!({}));
        let package = document["package"].as_object_mut().expect("package");
        match version {
            Some(version) => package.insert("version".to_owned(), version),
            None => package.remove("version"),
        };
        document
    };
    let mut digests = std::collections::BTreeSet::new();
    for (case, version) in [
        ("version 1.0.0", Some(json!("1.0.0"))),
        ("version 2.0.0", Some(json!("2.0.0"))),
        ("no version", None),
        ("a non-string version", Some(json!(7))),
        ("a null version", Some(Value::Null)),
    ] {
        let (package, evidence) = package_over(&versioned(version), "total");
        match read(&package, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{case}: expected admission, got {other:?}"),
        }
        digests.insert(package["lock"]["model_selections"][0]["digest"].to_string());
    }
    // Each document differs only in its version, so each is selected under
    // its own digest: the admissions above are five distinct bindings.
    assert_eq!(digests.len(), 5, "each version case is its own document");

    // A document naming another identity is refused at the row's identity,
    // whatever its version.
    let mut other = versioned(Some(json!("1.0.0")));
    other["package"]["identity"] = json!("acme/other");
    let (package, evidence) = package_over(&other, "total");
    assert_eq!(
        refused(&package, &evidence),
        refusal_cause(
            CheckedPackageRefusalCode::InvalidModelBinding,
            "/lock/model_selections/0/identity",
            CheckedPackageRefusalCause::WrongModelSelection
        )
    );
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

/// The bytes of [`domain_document`] with the JSON number text `number` at
/// `/package/<member>`, spelled exactly as given.
fn document_bytes_with_number(member: &str, number: &str) -> Vec<u8> {
    let mut document = domain_document(json!({}));
    document["package"][member] = json!(0);
    let text = String::from_utf8(canonical(&document)).expect("utf-8");
    let zero = format!("\"{member}\":0");
    assert_eq!(text.matches(&zero).count(), 1);
    text.replace(&zero, &format!("\"{member}\":{number}"))
        .into_bytes()
}

/// A read of a package whose one selection row names `digest`, with `bytes`
/// supplied under it.
fn read_selecting(digest: &str, bytes: &[u8]) -> CheckedPackageV2ReadResult {
    let package = package_selecting("total", digest);
    let mut evidence = evidence_for(&package);
    evidence.insert_domain_package_document(digest.to_owned(), bytes.to_vec());
    read(&package, &evidence)
}

fn number_refusal(
    document_pointer: &str,
    cause: CheckedPackageRefusalCause,
) -> CheckedPackageRefusal {
    CheckedPackageRefusal {
        document_pointer: Some(pointer(document_pointer)),
        ..refusal_cause(
            CheckedPackageRefusalCode::NoncanonicalWire,
            "/lock/model_selections/0/digest",
            cause,
        )
    }
}

/// Tracing: TC-048, FR-038-AC-93
#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_a_model_document_number_past_2_pow_53_refuses_with_its_document_pointer() {
    let other = "ab".repeat(32);
    for text in [
        "9007199254740993",
        "-9007199254740993",
        "9.007199254740993e15",
        "1e20",
        "18446744073709551617",
    ] {
        let bytes = document_bytes_with_number("count", text);
        // Under the document's own digest and under another one: the refusal
        // precedes `byte-digest-mismatch` either way.
        for digest in [sha256_hex(&bytes), other.clone()] {
            match read_selecting(&digest, &bytes) {
                CheckedPackageV2ReadResult::Refused(refused) => {
                    assert_eq!(
                        refused,
                        number_refusal(
                            "/package/count",
                            CheckedPackageRefusalCause::InexactInteger
                        ),
                        "{text}"
                    );
                }
                other => panic!("{text}: expected a refusal, read {other:?}"),
            }
        }
    }
}

/// Tracing: TC-048, FR-038-AC-93
#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_a_model_document_number_at_or_under_2_pow_53_is_digested() {
    for (text, canonical_text) in [
        ("9007199254740992", "9007199254740992"),
        ("-9007199254740992", "-9007199254740992"),
        ("9.007199254740992e15", "9007199254740992"),
        ("0.5", "0.5"),
    ] {
        let bytes = document_bytes_with_number("count", text);
        // The digest is of the canonical form of the number read.
        let digest = sha256_hex(&document_bytes_with_number("count", canonical_text));
        assert!(
            matches!(
                read_selecting(&digest, &bytes),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{text} is digested and admitted"
        );
    }
}

/// [`domain_document`] with an integer value type `Count` whose `max` bound is
/// `maximum`, a JSON number or a decimal string.
fn document_with_count_bound(maximum: Value) -> Value {
    let mut document = domain_document(json!({}));
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "count"},
            "construct": {"meaning": "quire.meaning.model.value-type/v1"},
        }));
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": "ix://acme/orders/Count", "displayName": "Count",
            "kind": {"module": "acme/orders", "name": "count"},
            "roles": [], "extensions": [], "unknownPolicy": "reject", "scalar": "integer",
            "constraints": [
                {"keyword": "min", "operands": {"value": 0}},
                {"keyword": "max", "operands": {"value": maximum}},
            ],
        }));
    document
}

/// Tracing: TC-048, FR-038-AC-93
#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_an_integer_value_type_bound_past_2_pow_53_is_refused_as_a_number() {
    // Written as a JSON number, the bound is refused at its own pointer, where
    // the reader admitted it before.
    let past = document_with_count_bound(json!(9_007_199_254_740_993_i64));
    let bytes = canonical(&past);
    match read_selecting(&sha256_hex(&bytes), &bytes) {
        CheckedPackageV2ReadResult::Refused(refused) => {
            assert_eq!(
                refused,
                number_refusal(
                    "/types/1/constraints/1/operands/value",
                    CheckedPackageRefusalCause::InexactInteger
                )
            );
        }
        other => panic!("expected a refusal, read {other:?}"),
    }
    // At 2^53 the same document is admitted.
    let at = document_with_count_bound(json!(9_007_199_254_740_992_i64));
    let bytes = canonical(&at);
    assert!(matches!(
        read_selecting(&sha256_hex(&bytes), &bytes),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    // Written as a decimal string, the bound is not a number of the document:
    // exact integers past 2^53 travel this way, so it is admitted.
    let spelled = document_with_count_bound(json!("9007199254740993"));
    let bytes = canonical(&spelled);
    assert!(matches!(
        read_selecting(&sha256_hex(&bytes), &bytes),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Tracing: TC-048, FR-038-AC-93
#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_the_first_number_in_document_order_past_2_pow_53_is_named() {
    let document = domain_document(json!({}));
    let text = String::from_utf8(canonical(&document)).expect("utf-8");
    // `/b` is written first and `/a/0` second, so document order is not the
    // sorted order that would name `/a/0` first.
    let spelled = format!(
        "{{\"b\":9007199254740993,\"a\":[9007199254740995],{}",
        &text[1..]
    );
    let bytes = spelled.into_bytes();
    match read_selecting(&sha256_hex(&bytes), &bytes) {
        CheckedPackageV2ReadResult::Refused(refused) => {
            assert_eq!(
                refused,
                number_refusal("/b", CheckedPackageRefusalCause::InexactInteger)
            );
        }
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// The JSON object prefix `members` placed before the members of a model
/// document's canonical text, so the numbers in `members` come first in
/// document order.
fn document_bytes_led_by(members: &str) -> Vec<u8> {
    let document = domain_document(json!({}));
    let text = String::from_utf8(canonical(&document)).expect("utf-8");
    format!("{{{members},{}", &text[1..]).into_bytes()
}

/// Tracing: TC-048, FR-038-AC-109
#[trace("TC-048", "FR-038-AC-109")]
#[test]
fn tc_048_a_model_document_number_with_no_exact_rfc_8785_spelling_refuses_inexact_number() {
    let other = "ab".repeat(32);
    for text in [
        "0.1000000000000000000001",
        "9007199254740993.5",
        "-0.1000000000000000000001",
        "4.9e-324",
        "1e-400",
        // The odd-digit spellings of three ties, whose even-digit spellings are
        // admitted below.
        "1125899906842624.3",
        "1500000000000000.3",
        "2.9802322387695313e-8",
    ] {
        let bytes = document_bytes_with_number("ratio", text);
        for digest in [sha256_hex(&bytes), other.clone()] {
            match read_selecting(&digest, &bytes) {
                CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                    refused,
                    number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactNumber),
                    "{text}"
                ),
                other => panic!("{text}: expected a refusal, read {other:?}"),
            }
        }
    }
}

/// Tracing: TC-048, FR-038-AC-109
#[trace("TC-048", "FR-038-AC-109")]
#[test]
fn tc_048_a_model_document_number_whose_value_its_encoding_keeps_is_digested() {
    // Each pair is the number as spelled and the text `quire-canonical` writes
    // for it: the value is the same, so the spelling does not matter.
    for (text, canonical_text) in [
        ("0.1", "0.1"),
        ("0.5", "0.5"),
        ("1.5", "1.5"),
        ("-0.25", "-0.25"),
        ("5e-324", "5e-324"),
        ("2.5e-10", "2.5e-10"),
        ("1.0", "1"),
        ("-0", "0"),
        ("1e2", "100"),
        ("1125899906842624.2", "1125899906842624.2"),
        ("1500000000000000.2", "1500000000000000.2"),
        ("2.9802322387695312e-8", "2.9802322387695312e-8"),
    ] {
        let bytes = document_bytes_with_number("ratio", text);
        let digest = sha256_hex(&document_bytes_with_number("ratio", canonical_text));
        assert!(
            matches!(
                read_selecting(&digest, &bytes),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{text} is digested and admitted"
        );
    }
}

/// Tracing: TC-048, FR-038-AC-110
#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_the_first_inexact_number_in_document_order_is_named_with_its_own_cause() {
    // An inexact number at `/b` before a whole number past 2^53 at `/a/0`
    // names `/b`; the reverse order names the whole number. `9007199254740992.5`
    // is not whole, so it is never an integer cause though its nearest double
    // is 2^53.
    for (members, pointer, cause) in [
        (
            "\"b\":9007199254740992.5",
            "/b",
            CheckedPackageRefusalCause::InexactNumber,
        ),
        (
            "\"b\":0.1000000000000000000001,\"a\":[9007199254740993]",
            "/b",
            CheckedPackageRefusalCause::InexactNumber,
        ),
        (
            "\"b\":9007199254740993,\"a\":[0.1000000000000000000001]",
            "/b",
            CheckedPackageRefusalCause::InexactInteger,
        ),
        (
            "\"a\":[9007199254740993.0],\"b\":0.1000000000000000000001",
            "/a/0",
            CheckedPackageRefusalCause::InexactInteger,
        ),
    ] {
        let bytes = document_bytes_led_by(members);
        match read_selecting(&sha256_hex(&bytes), &bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => {
                assert_eq!(refused, number_refusal(pointer, cause), "{members}");
            }
            other => panic!("{members}: expected a refusal, read {other:?}"),
        }
    }
}

/// Tracing: TC-048, FR-038-AC-110
#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_a_number_past_the_double_range_refuses_inexact_integer_with_its_pointer() {
    let other = "ab".repeat(32);
    for text in ["1e400", "-1e400", "1e309"] {
        // Nested: at a member of the model document, under its own digest and
        // another, so the refusal precedes `byte-digest-mismatch`.
        let bytes = document_bytes_with_number("ratio", text);
        for digest in [sha256_hex(&bytes), other.clone()] {
            match read_selecting(&digest, &bytes) {
                CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                    refused,
                    number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactInteger),
                    "{text}"
                ),
                other => panic!("{text}: expected a refusal, read {other:?}"),
            }
        }
        // In an array, so the pointer carries an index.
        let bytes = document_bytes_led_by(&format!("\"a\":[0,{text}]"));
        match read_selecting(&sha256_hex(&bytes), &bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("/a/1", CheckedPackageRefusalCause::InexactInteger),
                "{text}"
            ),
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }
        // At the top level the document is the number and the pointer is empty.
        let bytes = text.as_bytes().to_vec();
        match read_selecting(&sha256_hex(&bytes), &bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("", CheckedPackageRefusalCause::InexactInteger),
                "{text}"
            ),
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }
    }
}

/// The refusal of a model document the reader does not parse: the digest is
/// taken over the raw bytes, so it matches no `sha256-jcs` digest.
fn raw_digest_refusal() -> CheckedPackageRefusal {
    refusal_cause(
        CheckedPackageRefusalCode::StaleDependency,
        "/lock/model_selections/0/digest",
        CheckedPackageRefusalCause::ByteDigestMismatch,
    )
}

/// Tracing: TC-048, FR-038-AC-110
#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_the_first_reader_fault_decides_when_faults_coexist() {
    // QSL FR-056: the first fault `quire_canonical::read` returns decides. A
    // number with no finite double is raised when it is read; a repeated
    // member name only when its object closes. Hand-written expectations.
    let number = |pointer: &str, cause| number_refusal(pointer, cause);
    let led = |members: &str| document_bytes_led_by(members);
    let cases: [(Vec<u8>, CheckedPackageRefusal); 11] = [
        // The reader checks that the whole input is UTF-8 before it reads any
        // value, so invalid UTF-8 after the number is the first fault.
        (b"[1e400,\"\xFF\"]".to_vec(), raw_digest_refusal()),
        // A lone surrogate escape is found when the string is read, after the
        // number.
        (
            b"[1e400,\"\\ud800\"]".to_vec(),
            number("/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        // The out-of-range number is read before the object closes, so it is
        // named ahead of the earlier repeated name.
        (
            led("\"a\":1,\"a\":2,\"n\":1e400"),
            number("/n", CheckedPackageRefusalCause::InexactInteger),
        ),
        // The repeated name is in an object that closes before `1e400`, so it
        // is the first fault and the bytes take the raw-digest path.
        (b"[{\"a\":1,\"a\":2},1e400]".to_vec(), raw_digest_refusal()),
        // An inexact-number-to-be (`1e-400` reads as zero) is no reader
        // fault, so the repeated name is the first fault.
        (led("\"a\":1,\"a\":2,\"n\":1e-400"), raw_digest_refusal()),
        // Truncation after the number: the number is read first.
        (
            b"[1e400".to_vec(),
            number("/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        // The number before the repeated name, in the same object.
        (
            led("\"n\":1e400,\"a\":1,\"a\":2"),
            number("/n", CheckedPackageRefusalCause::InexactInteger),
        ),
        // An out-of-range number is named ahead of an earlier inexact number
        // (a reader-decided one is refused only after the read succeeds).
        (
            led("\"b\":0.1000000000000000000001,\"a\":[1e400]"),
            number("/a/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        (
            led("\"b\":9007199254740993,\"a\":[1e400]"),
            number("/a/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        (
            led("\"b\":9007199254740993,\"a\":[-1e400]"),
            number("/a/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        // An out-of-range number that is not whole is an inexact number.
        (
            led(&format!(
                "\"b\":9007199254740993,\"a\":[{}.5]",
                "9".repeat(400)
            )),
            number("/a/0", CheckedPackageRefusalCause::InexactNumber),
        ),
    ];
    let other = "ab".repeat(32);
    for (bytes, expected) in cases {
        let shown = String::from_utf8_lossy(&bytes)
            .chars()
            .take(60)
            .collect::<String>();
        // The raw-path outcome is pinned under a foreign digest only: under the
        // document's own raw digest QSL FR-056 says `wrong-model-selection`,
        // which this reader does not yet do (IR-578).
        let own = sha256_hex(&bytes);
        let digests = if expected == raw_digest_refusal() {
            vec![other.clone()]
        } else {
            vec![own, other.clone()]
        };
        for digest in digests {
            match read_selecting(&digest, &bytes) {
                CheckedPackageV2ReadResult::Refused(refused) => {
                    assert_eq!(refused, expected, "{shown}");
                }
                other => panic!("{shown}: expected a refusal, read {other:?}"),
            }
        }
    }
}

/// Tracing: TC-048, FR-038-AC-110
#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_a_non_whole_number_past_the_double_range_refuses_inexact_number() {
    // 400 nines then `.5`: past the double range, but not a whole value, so
    // the cause is `inexact-number`, never `inexact-integer`.
    let text = format!("{}.5", "9".repeat(400));
    for number in [text.clone(), format!("-{text}")] {
        let bytes = document_bytes_with_number("ratio", &number);
        match read_selecting(&sha256_hex(&bytes), &bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactNumber),
            ),
            other => panic!("expected a refusal, read {other:?}"),
        }
    }
}

/// Tracing: TC-048, FR-038-AC-109
#[trace("TC-048", "FR-038-AC-109")]
#[test]
fn tc_048_a_number_below_the_double_range_refuses_inexact_number_on_its_text() {
    // These read as zero with their text kept, so the rule is IR's own, on
    // the text: zero is not their exact value.
    let other = "ab".repeat(32);
    for text in ["1e-400", "-1e-400"] {
        let bytes = document_bytes_with_number("ratio", text);
        for digest in [sha256_hex(&bytes), other.clone()] {
            match read_selecting(&digest, &bytes) {
                CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                    refused,
                    number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactNumber),
                    "{text}"
                ),
                other => panic!("{text}: expected a refusal, read {other:?}"),
            }
        }
    }
}

/// Tracing: TC-048, FR-038-AC-93
#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_a_package_stream_refusal_carries_no_document_pointer() {
    let package = package_selecting("total", &"ab".repeat(32));
    let mut bytes = canonical(&package);
    bytes.push(b' ');
    match CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    ) {
        CheckedPackageV2ReadResult::Refused(refused) => {
            assert_eq!(
                refused,
                refusal_bytes(CheckedPackageRefusalCode::NoncanonicalWire)
            );
            assert_eq!(refused.document_pointer, None);
        }
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// Tracing: TC-048, FR-038-AC-94
#[trace("TC-048", "FR-038-AC-94")]
#[test]
fn tc_048_a_model_document_is_read_and_encoded_under_the_byte_limit() {
    let mut document = domain_document(json!({}));
    document["package"]["padding"] = json!("x".repeat(30_000));
    let bytes = canonical(&document);
    let digest = sha256_hex(&bytes);
    let length = u64::try_from(bytes.len()).expect("length");
    let package = package_selecting("total", &digest);
    let package_bytes = canonical(&package);
    assert!(
        u64::try_from(package_bytes.len()).expect("length") < length,
        "the package is shorter than the document"
    );
    let mut evidence = evidence_for(&package);
    evidence.insert_domain_package_document(digest, bytes);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.bytes = length;
    assert!(
        matches!(
            CheckedPackageV2::read(&package_bytes, limits, &evidence),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "a document exactly `limits.bytes` long is read, digested and matches"
    );
    limits.bytes = length - 1;
    match CheckedPackageV2::read(&package_bytes, limits, &evidence) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Bytes);
            assert_eq!(incomplete.limit, length - 1);
            assert_eq!(incomplete.consumed, length);
            assert_eq!(incomplete.path, None);
        }
        other => panic!("expected an incomplete read, got {other:?}"),
    }
    // The work limit charged for the same document is located at the row.
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = length.div_ceil(1024) - 1;
    match CheckedPackageV2::read(&package_bytes, limits, &evidence) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                incomplete.path.map(|path| path.to_string()).as_deref(),
                Some("/lock/model_selections/0")
            );
        }
        other => panic!("expected an incomplete read, got {other:?}"),
    }
}

const SUB: &str = "ix://acme/orders/Sub";
const SUBSUB: &str = "ix://acme/orders/SubSub";
const INVOICE: &str = "ix://acme/orders/Invoice";
const TAGGED: &str = "ix://acme/orders/Tagged";
const CODED: &str = "ix://acme/orders/Coded";
const BOTH: &str = "ix://acme/orders/Both";
const MODELS: [&str; 7] = [ORDER, SUB, SUBSUB, INVOICE, TAGGED, CODED, BOTH];
/// A domain package identity the lock never selects.
const OTHER_PACKAGE: &str = "acme/other";

fn edge_field(
    owner: &str,
    name: &str,
    type_ref: &str,
    presence: &str,
    multiplicity: Value,
) -> Value {
    json!({
        "identity": format!("{owner}/{name}"), "name": name, "typeRef": type_ref,
        "presence": presence, "nullable": false, "defaultKind": "none",
        "multiplicity": multiplicity,
    })
}

fn object_type_declaration(node: &str, supertypes: &[&str], fields: Vec<Value>) -> Value {
    json!({
        "identity": node, "displayName": node,
        "kind": {"module": "acme/orders", "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": fields, "operations": [],
    })
}

/// A document whose `Order` declares one edge-shaped and several
/// non-edge-shaped fields; `Sub` specializes `Order` and declares an
/// optional `Sub` edge, `SubSub` specializes `Sub`, `Invoice` is unrelated,
/// and `Both` inherits a `code` field from each of `Tagged` and `Coded`.
fn edge_document() -> Value {
    let one = json!({"lower": 1, "upper": 1, "ordered": false, "unique": true});
    let order = |name: &str, presence: &str, multiplicity: Value| {
        edge_field(ORDER, name, ORDER, presence, multiplicity)
    };
    let fields = vec![
        order("parent", "optional", one.clone()),
        order("origin", "required", one.clone()),
        order(
            "prior",
            "required",
            json!({"lower": 0, "upper": 3, "ordered": true, "unique": false}),
        ),
        order(
            "peers",
            "required",
            json!({"lower": 0, "upper": 3, "ordered": false, "unique": true}),
        ),
        order(
            "trail",
            "required",
            json!({"lower": 0, "ordered": true, "unique": false}),
        ),
        edge_field(ORDER, "invoice", INVOICE, "required", one.clone()),
        edge_field(
            ORDER,
            "total",
            "ix://quire/native/Integer",
            "required",
            one.clone(),
        ),
    ];
    let code = |owner: &str| {
        vec![edge_field(
            owner,
            "code",
            "ix://quire/native/Integer",
            "required",
            one.clone(),
        )]
    };
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": IDENTITY, "version": VERSION},
        "constructs": [{
            "kind": {"module": "acme/orders", "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [
            object_type_declaration(ORDER, &[], fields),
            object_type_declaration(
                SUB,
                &[ORDER],
                vec![edge_field(SUB, "child", SUB, "optional", one.clone())],
            ),
            object_type_declaration(SUBSUB, &[SUB], vec![]),
            object_type_declaration(INVOICE, &[], vec![]),
            object_type_declaration(TAGGED, &[], code(TAGGED)),
            object_type_declaration(CODED, &[], code(CODED)),
            object_type_declaration(BOTH, &[TAGGED, CODED], vec![]),
        ],
    })
}

/// The model declaration node key of `node` owned by the domain package
/// `identity`; the owner is content-only and carries no version.
fn declaration_key_of(node: &str, identity: &str) -> String {
    structural_key(
        "model",
        "object_type",
        None,
        Some(json!({"kind": "model", "identity": identity, "node": node})),
        &empty(),
    )
}

/// What an operand of [`reaches_package`] is typed as.
#[derive(Clone, Copy)]
enum Operand {
    /// `Reference<node>`.
    Reference(&'static str),
    /// `node` itself, an object.
    Object(&'static str),
}

/// A package over [`edge_document`] holding `reaches(source, target, name)`
/// as a `quire.op.model.reaches_field` application whose member declaration
/// is `declaring` (keyed under the domain package `package`), with the
/// operands typed as given. Returns the package, its evidence, and the
/// application's node position.
fn reaches_package(
    (declaring, package_identity): (&str, &str),
    name: &str,
    source: Operand,
    target: Operand,
) -> (Value, quire_contract_ir::CheckedPackageEvidence, usize) {
    let document = edge_document();
    let digest = sha256_hex(&canonical(&document));
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": IDENTITY,
        "digest_domain": "sha256-jcs", "digest": digest,
    }]);

    let boolean = structural_key("scalar_type", "boolean", None, None, &empty());
    let mut models: Vec<(String, String)> = MODELS
        .into_iter()
        .map(|node| (node.to_owned(), declaration_key_of(node, IDENTITY)))
        .collect();
    if package_identity == OTHER_PACKAGE
        || matches!(source, Operand::Reference("unselected"))
        || matches!(target, Operand::Reference("unselected"))
    {
        models.push((
            "unselected".to_owned(),
            declaration_key_of(ORDER, OTHER_PACKAGE),
        ));
    }
    let reference_types: Vec<(String, String, Value)> = models
        .iter()
        .map(|(node, key)| {
            let body = json!({"term": "aggregate", "members": [reference(key)]});
            let type_key = structural_key("composite_type", "reference", None, None, &body);
            (node.clone(), type_key, body)
        })
        .collect();
    let model_of = |node: &str, owner: &str| {
        let name = if owner == IDENTITY {
            node
        } else {
            "unselected"
        };
        models
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, key)| key.clone())
            .expect("declared model")
    };
    // A reference operand is a parameter of the reference type; an object
    // operand names the model declaration node itself.
    let operand = |operand: Operand, name: &str, level: &str| match operand {
        Operand::Reference(node) => {
            let type_key = reference_types
                .iter()
                .find(|(candidate, _, _)| candidate == node)
                .map(|(_, key, _)| key.clone())
                .expect("reference type");
            let (key, node) = parameter(name, level, &type_key);
            (key, Some(node))
        }
        Operand::Object(node) => (model_of(node, IDENTITY), None),
    };
    let (receiver, receiver_node) = operand(source, "receiver", "0");
    let (argument, argument_node) = operand(target, "argument", "1");
    let declaring_key = model_of(declaring, package_identity);
    let body = json!({
        "term": "application",
        "operator": "reaches",
        "operation": {
            "identity": "quire.op.model.reaches_field", "laws": [], "mode": null,
            "member": {"kind": "field", "declaration": node_id(&declaring_key), "name": name},
            "leaves": [],
        },
        "result_type": node_id(&boolean),
        "arguments": [reference(&receiver), reference(&argument)],
    });
    let reaches = sha256_hex(&canonical(&json!({
        "version": APPLICATION_NODE,
        "node_tag": "expression",
        "semantic_form": "reachability",
        "semantic_type": node_id(&boolean),
        "declaration": null,
        "recursion": null,
        "body": body,
    })));
    let mut dependencies = [receiver.as_str(), argument.as_str(), declaring_key.as_str()];
    dependencies.sort_unstable();
    let dependencies: Vec<&str> = {
        let mut unique = dependencies.to_vec();
        unique.dedup();
        unique
    };
    let mut nodes = vec![
        wire_node(&INTEGER, "scalar_type", "integer", &INTEGER, &[], empty()),
        wire_node(&TEXT, "scalar_type", "text", &TEXT, &[], empty()),
        wire_node(&boolean, "scalar_type", "boolean", &boolean, &[], empty()),
    ];
    for (declared, key) in &models {
        let mut node = wire_node(key, "model", "object_type", key, &[], empty());
        let (identity, model_node) = if declared == "unselected" {
            (OTHER_PACKAGE, ORDER)
        } else {
            (IDENTITY, declared.as_str())
        };
        node["owner"] = json!({"kind": "model", "identity": identity, "node": model_node});
        nodes.push(node);
    }
    for ((_, model), (_, type_key, type_body)) in models.iter().zip(&reference_types) {
        nodes.push(wire_node(
            type_key,
            "composite_type",
            "reference",
            type_key,
            &[model.as_str()],
            type_body.clone(),
        ));
    }
    nodes.extend(receiver_node.into_iter().chain(argument_node));
    nodes.push(wire_node(
        &reaches,
        "expression",
        "reachability",
        &boolean,
        &dependencies,
        body,
    ));
    let position = nodes.len() - 1;
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let mut evidence = evidence_for(&package);
    evidence.insert_domain_package_document(digest, canonical(&document));
    (package, evidence, position)
}

const SELECTED: &str = IDENTITY;

fn reaches(
    declaring: &'static str,
    name: &str,
    source: &'static str,
    target: &'static str,
) -> (Value, quire_contract_ir::CheckedPackageEvidence, usize) {
    reaches_package(
        (declaring, SELECTED),
        name,
        Operand::Reference(source),
        Operand::Reference(target),
    )
}

/// FR-322 "Reaches over a field": an optional, a required and a bounded
/// sequence reference to the owner are edges, including through a subtype
/// declaring node, a subtype target and a target two levels down.
///
/// Tracing: TC-056, FR-040-AC-13
#[trace("TC-056", "FR-040-AC-13")]
#[test]
fn tc_056_reaches_field_admits_each_reference_edge_shape() {
    let admitted = [
        (ORDER, "parent", ORDER, ORDER),
        (ORDER, "prior", ORDER, ORDER),
        (ORDER, "origin", ORDER, ORDER),
        (SUB, "parent", SUB, ORDER),
        (ORDER, "parent", ORDER, SUB),
        (ORDER, "parent", ORDER, SUBSUB),
        (SUB, "child", SUB, SUBSUB),
    ];
    for (declaring, name, source, target) in admitted {
        let (package, evidence, _) = reaches(declaring, name, source, target);
        match read(&package, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{name} over {source}/{target}: expected admission, got {other:?}"),
        }
    }
}

/// The package reader's relationship-edge check resolves `parent` on each
/// declaring type. `Sub` inherits `Order`'s seven fields and declares `child`,
/// so its field table has one more entry than `Order`'s table.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_relationship_edge_read_charges_its_effective_table_end_to_end() {
    let smallest = |declaring, source| {
        let (package, evidence, _) = reaches(declaring, "parent", source, ORDER);
        let read = |work| {
            let mut limits = CheckedPackageReadLimits::bounded();
            limits.work = work;
            CheckedPackageV2::read(&canonical(&package), limits, &evidence)
        };
        let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
        assert!(matches!(
            read(high),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        while low < high {
            let middle = low + (high - low) / 2;
            match read(middle) {
                CheckedPackageV2ReadResult::Admitted(_) => high = middle,
                CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
                other => panic!("expected admission or work limit, got {other:?}"),
            }
        }
        assert!(matches!(
            read(low - 1),
            CheckedPackageV2ReadResult::Incomplete(_)
        ));
        low
    };
    assert_eq!(smallest(SUB, SUB), smallest(ORDER, ORDER) + 1);
}

/// FR-322 "Reaches over a field": a field that is not a reference edge to
/// its owner, an operand that does not name the declaring node, a target
/// that is not a subtype of the edge's owner (a supertype included), an
/// object operand, an undeclared field, an ambiguous field name and an
/// unselected declaration each refuse with their own code at the operand or
/// member at fault.
///
/// Tracing: TC-056, FR-040-AC-13
#[trace("TC-056", "FR-040-AC-13")]
#[test]
fn tc_056_reaches_field_refuses_an_invalid_edge_where_it_fails() {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;
    let ineligible = (Code::IllTyped, Cause::OperatorIneligible);
    let name = "body/operation/member/name";
    let first = "body/arguments/0";
    let second = "body/arguments/1";
    let by_reference = |declaring, member, source, target| {
        reaches_package(
            (declaring, SELECTED),
            member,
            Operand::Reference(source),
            Operand::Reference(target),
        )
    };
    let cases = [
        (by_reference(ORDER, "peers", ORDER, ORDER), ineligible, name),
        (by_reference(ORDER, "trail", ORDER, ORDER), ineligible, name),
        (
            by_reference(ORDER, "invoice", ORDER, ORDER),
            ineligible,
            name,
        ),
        (by_reference(ORDER, "total", ORDER, ORDER), ineligible, name),
        (
            by_reference(ORDER, "missing", ORDER, ORDER),
            ineligible,
            name,
        ),
        (
            by_reference(ORDER, "parent", ORDER, INVOICE),
            ineligible,
            second,
        ),
        (by_reference(ORDER, "parent", SUB, ORDER), ineligible, first),
        // An edge to `Sub` does not reach an `Order`, a supertype of its owner.
        (by_reference(SUB, "child", SUB, ORDER), ineligible, second),
        (
            reaches_package(
                (ORDER, SELECTED),
                "parent",
                Operand::Reference(ORDER),
                Operand::Object(ORDER),
            ),
            ineligible,
            second,
        ),
        (
            by_reference(BOTH, "code", BOTH, BOTH),
            (Code::AmbiguousDeclaration, Cause::AmbiguousName),
            name,
        ),
    ];
    for ((package, evidence, position), (code, cause), path) in cases {
        let refusal = refused(&package, &evidence);
        let expected = format!("/semantic_graph/nodes/{position}/{path}");
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some(expected.as_str()),
            "{refusal:?}"
        );
        assert_eq!(
            (refusal.code, refusal.cause),
            (code, Some(cause)),
            "{expected}"
        );
    }
    // An unselected owner is now refused at its own node before the
    // application can inspect the member that names it (FR-038-AC-155).
    let (package, evidence, _) = reaches_package(
        (ORDER, OTHER_PACKAGE),
        "parent",
        Operand::Reference("unselected"),
        Operand::Reference("unselected"),
    );
    let unselected = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["owner"]["identity"] == OTHER_PACKAGE)
        .expect("unselected owner");
    let refusal = refused(&package, &evidence);
    assert_eq!(refusal.code, Code::MissingDeclaration);
    assert_eq!(refusal.cause, Some(Cause::MissingSelection));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some(format!("/semantic_graph/nodes/{unselected}/node_id").as_str())
    );
}
