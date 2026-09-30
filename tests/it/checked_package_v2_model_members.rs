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
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": IDENTITY, "version": VERSION,
        "digest_domain": "sha256-jcs", "digest": digest,
    }]);

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
    let nodes = [
        wire_node(&INTEGER, "scalar_type", "integer", &INTEGER, &[], empty()),
        wire_node(&TEXT, "scalar_type", "text", &TEXT, &[], empty()),
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

const SUB: &str = "ix://acme/orders/Sub";
const SUBSUB: &str = "ix://acme/orders/SubSub";
const INVOICE: &str = "ix://acme/orders/Invoice";
const TAGGED: &str = "ix://acme/orders/Tagged";
const CODED: &str = "ix://acme/orders/Coded";
const BOTH: &str = "ix://acme/orders/Both";
const MODELS: [&str; 7] = [ORDER, SUB, SUBSUB, INVOICE, TAGGED, CODED, BOTH];
const OTHER_VERSION: &str = "2.0.0";

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

fn declaration_key_of(node: &str, version: &str) -> String {
    structural_key(
        "model",
        "object_type",
        None,
        Some(json!({"kind": "model", "identity": IDENTITY, "version": version, "node": node})),
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
/// is `declaring` (keyed under `version`), with the operands typed as
/// given. Returns the package, its evidence, and the application's node
/// position.
fn reaches_package(
    (declaring, version): (&str, &str),
    name: &str,
    source: Operand,
    target: Operand,
) -> (Value, quire_contract_ir::CheckedPackageEvidence, usize) {
    let document = edge_document();
    let digest = sha256_hex(&canonical(&document));
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": IDENTITY, "version": VERSION,
        "digest_domain": "sha256-jcs", "digest": digest,
    }]);

    let boolean = structural_key("scalar_type", "boolean", None, None, &empty());
    let mut models: Vec<(String, String)> = MODELS
        .into_iter()
        .map(|node| (node.to_owned(), declaration_key_of(node, VERSION)))
        .collect();
    models.push((
        "unselected".to_owned(),
        declaration_key_of(ORDER, OTHER_VERSION),
    ));
    let reference_types: Vec<(String, String, Value)> = models
        .iter()
        .map(|(node, key)| {
            let body = json!({"term": "aggregate", "members": [reference(key)]});
            let type_key = structural_key("composite_type", "reference", None, None, &body);
            (node.clone(), type_key, body)
        })
        .collect();
    let model_of = |node: &str, version: &str| {
        let name = if version == VERSION {
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
        Operand::Object(node) => (model_of(node, VERSION), None),
    };
    let (receiver, receiver_node) = operand(source, "receiver", "0");
    let (argument, argument_node) = operand(target, "argument", "1");
    let declaring_key = model_of(declaring, version);
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
    for (_, key) in &models {
        nodes.push(wire_node(key, "model", "object_type", key, &[], empty()));
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

const SELECTED: &str = VERSION;

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
    let declaration = "body/operation/member/declaration";
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
        (
            reaches_package(
                (ORDER, OTHER_VERSION),
                "parent",
                Operand::Reference("unselected"),
                Operand::Reference("unselected"),
            ),
            (Code::MissingDeclaration, Cause::MissingSelection),
            declaration,
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
}
