// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-628: `CheckedPackageV2::model_object_fields` over packages the reader
//! admits, each built here node by node with every key recomputed through
//! `quire-canonical` from the preimages of FR-322 and QSL FR-092, and over a
//! real Semantic IR 2.0.0 domain package document the lock selects. Nothing of
//! the reader's own derivation is used to key a node.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_package, rebuild_source_map, refresh_identity,
    sha256_hex,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedCollectionKind, CheckedMemberType, CheckedModelFieldsError, CheckedModelObjectFields,
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const IDENTITY: &str = "acme/orders";
const STRUCTURAL_NODE: &str = "quire.structural-node/v1";
const APPLICATION_NODE: &str = "quire.application-node/v1";
const OBJECT: &str = "object_type";
const INTERFACE: &str = "systems_interface";

fn orders(short: &str) -> String {
    format!("ix://acme/orders/{short}")
}

/// The SHA-256 of the canonical bytes `quire-canonical` writes for `preimage`.
fn key_of(preimage: &Value) -> String {
    quire_canonical::sha256(preimage, quire_canonical::Limits::new(1 << 20))
        .expect("a preimage of this size encodes")
        .to_string()
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn structural(
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
    key_of(&preimage)
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

/// The model declaration node key of `short`, a declaration of the selected
/// domain package, in `form`.
fn declaration_key(short: &str, form: &str) -> String {
    let tag = if form == "relationship" {
        "relation"
    } else {
        "model"
    };
    structural(
        tag,
        form,
        None,
        Some(json!({"kind": "model", "identity": IDENTITY, "node": orders(short)})),
        &empty(),
    )
}

/// A member type, as a test states it: a node key is derived from it here.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Ty {
    Boolean,
    Integer,
    Range(i128, i128),
    /// `Reference<O>`, by the key of `O`'s model declaration node.
    Reference(String),
    Option(Box<Ty>),
    Collection(Kind, Box<Ty>, Option<(u64, u64)>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Set,
    Bag,
    Sequence,
    OrderedSet,
}

impl Kind {
    fn form(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Bag => "bag",
            Self::Sequence => "sequence",
            Self::OrderedSet => "ordered_set",
        }
    }
}

fn ty_of(member: &CheckedMemberType) -> Ty {
    match member {
        CheckedMemberType::Boolean => Ty::Boolean,
        CheckedMemberType::Integer => Ty::Integer,
        CheckedMemberType::IntRange { lower, upper } => Ty::Range(*lower, *upper),
        CheckedMemberType::Reference(target) => Ty::Reference(target.digest.to_string()),
        CheckedMemberType::Option(element) => Ty::Option(Box::new(ty_of(element))),
        CheckedMemberType::Collection {
            kind,
            element,
            bounds,
        } => Ty::Collection(
            match kind {
                CheckedCollectionKind::Set => Kind::Set,
                CheckedCollectionKind::Bag => Kind::Bag,
                CheckedCollectionKind::Sequence => Kind::Sequence,
                CheckedCollectionKind::OrderedSet => Kind::OrderedSet,
            },
            Box::new(ty_of(element)),
            *bounds,
        ),
    }
}

fn bounds_body(integer: &str, lower: &str, upper: &str) -> Value {
    let binding = |name: &str, value: &str| {
        json!({"term": "binding", "name": name, "value": {
            "term": "literal", "type": node_id(integer), "value_kind": "integer", "value": value,
        }})
    };
    json!({"term": "aggregate", "members": [binding("min", lower), binding("max", upper)]})
}

/// Adds the nodes of `ty` to `nodes`, each keyed from its own preimage, and
/// returns the key of the type.
fn type_nodes(ty: &Ty, nodes: &mut BTreeMap<String, Value>) -> String {
    let mut add = |key: String, node: Value| {
        nodes.entry(key.clone()).or_insert(node);
        key
    };
    match ty {
        Ty::Boolean | Ty::Integer => {
            let form = if *ty == Ty::Boolean {
                "boolean"
            } else {
                "integer"
            };
            let key = structural("scalar_type", form, None, None, &empty());
            let node = wire_node(&key, "scalar_type", form, &key, &[], empty());
            add(key, node)
        }
        Ty::Range(lower, upper) => {
            let integer = type_nodes(&Ty::Integer, nodes);
            let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
            let key = structural(
                "bounded_domain",
                "integer_range",
                Some(&integer),
                None,
                &body,
            );
            let node = wire_node(
                &key,
                "bounded_domain",
                "integer_range",
                &integer,
                &[&integer],
                body,
            );
            nodes.entry(key.clone()).or_insert(node);
            key
        }
        Ty::Reference(target) => {
            let body = json!({"term": "aggregate", "members": [reference(target)]});
            let key = structural("composite_type", "reference", None, None, &body);
            let node = wire_node(&key, "composite_type", "reference", &key, &[target], body);
            add(key, node)
        }
        Ty::Option(element) => {
            let element = type_nodes(element, nodes);
            let body = json!({"term": "aggregate", "members": [reference(&element)]});
            let key = structural("composite_type", "option", None, None, &body);
            let node = wire_node(&key, "composite_type", "option", &key, &[&element], body);
            nodes.entry(key.clone()).or_insert(node);
            key
        }
        Ty::Collection(kind, element, bounds) => {
            let element = type_nodes(element, nodes);
            let body = json!({"term": "aggregate", "members": [reference(&element)]});
            let collection = structural("composite_type", kind.form(), None, None, &body);
            let node = wire_node(
                &collection,
                "composite_type",
                kind.form(),
                &collection,
                &[&element],
                body,
            );
            nodes.entry(collection.clone()).or_insert(node);
            let Some((lower, upper)) = bounds else {
                return collection;
            };
            let integer = type_nodes(&Ty::Integer, nodes);
            let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
            let key = structural(
                "bounded_domain",
                "collection_bounds",
                Some(&collection),
                None,
                &body,
            );
            let node = wire_node(
                &key,
                "bounded_domain",
                "collection_bounds",
                &collection,
                &[&collection],
                body,
            );
            nodes.entry(key.clone()).or_insert(node);
            key
        }
    }
}

// --- the domain package document -------------------------------------------------

/// `(lower, upper, ordered, unique)` of a declared multiplicity; `upper`
/// `None` is unbounded.
type Multiplicity = (u64, Option<u64>, bool, bool);

const ONE: Multiplicity = (1, Some(1), false, true);

struct Field {
    name: &'static str,
    type_ref: String,
    optional: bool,
    multiplicity: Multiplicity,
    redefines: Option<String>,
}

fn field(name: &'static str, type_ref: &str) -> Field {
    Field {
        name,
        type_ref: type_ref.to_owned(),
        optional: false,
        multiplicity: ONE,
        redefines: None,
    }
}

impl Field {
    fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    fn many(mut self, multiplicity: Multiplicity) -> Self {
        self.multiplicity = multiplicity;
        self
    }

    fn redefining(mut self, member: &str) -> Self {
        self.redefines = Some(member.to_owned());
        self
    }
}

const NATIVE: &str = "ix://quire/native/";

fn object(short: &str, supertypes: &[&str], fields: &[Field]) -> Value {
    typed_object(short, "entity", supertypes, fields)
}

fn typed_object(short: &str, kind: &str, supertypes: &[&str], fields: &[Field]) -> Value {
    let node = orders(short);
    let fields: Vec<Value> = fields
        .iter()
        .map(|field| {
            let (lower, upper, ordered, unique) = field.multiplicity;
            let mut multiplicity = json!({"lower": lower, "ordered": ordered, "unique": unique});
            if let Some(upper) = upper {
                multiplicity["upper"] = json!(upper);
            }
            let mut declared = json!({
                "identity": format!("{node}/{}", field.name), "name": field.name,
                "typeRef": field.type_ref,
                "presence": if field.optional { "optional" } else { "required" },
                "nullable": false, "defaultKind": "none", "multiplicity": multiplicity,
            });
            if let Some(target) = &field.redefines {
                declared["redefines"] = json!(target);
            }
            declared
        })
        .collect();
    json!({
        "identity": node, "displayName": short,
        "kind": {"module": "acme/orders", "name": kind},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes.iter().map(|short| orders(short)).collect::<Vec<_>>(),
        "fields": fields, "operations": [],
    })
}

/// A value type of `scalar` with `min` and `max` constraints, written as the
/// decimal text a bound past 2^53 needs.
fn value_type(short: &str, scalar: &str, bounds: Option<(&str, &str)>) -> Value {
    let constraints: Vec<Value> = bounds
        .into_iter()
        .flat_map(|(min, max)| {
            [
                json!({"keyword": "min", "operands": {"value": min}}),
                json!({"keyword": "max", "operands": {"value": max}}),
            ]
        })
        .collect();
    json!({
        "identity": orders(short), "displayName": short,
        "kind": {"module": "acme/orders", "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": scalar, "constraints": constraints,
    })
}

fn document(types: Vec<Value>) -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": IDENTITY, "version": "1.0.0"},
        "constructs": [
            {"kind": {"module": "acme/orders", "name": "entity"},
             "construct": {"meaning": "quire.meaning.model.object-type/v1"}},
            {"kind": {"module": "acme/orders", "name": "digit"},
             "construct": {"meaning": "quire.meaning.model.value-type/v1"}},
            {"kind": {"module": "acme/orders", "name": "interface"},
             "construct": {"meaning": "quire.meaning.systems.interface/v1"}},
            {"kind": {"module": "acme/orders", "name": "event"},
             "construct": {"meaning": "quire.meaning.model.event-type/v1"}},
        ],
        "types": types,
    })
}

fn thousand() -> Value {
    value_type("Thousand", "integer", Some(("0", "1000")))
}

// --- the package ------------------------------------------------------------------

/// A read of `field` on `owner`, typed `ty`: an `expression`/`query`
/// application of `quire.op.record.project` over the owner's declaration node.
struct Read {
    owner: &'static str,
    field: &'static str,
    ty: Ty,
}

/// A package selecting `document`, with a model declaration node for each of
/// `declared` (`(short name, form)`) and one read node per entry of `reads`;
/// returns the package, its evidence and the position of each read.
fn package(
    document: &Value,
    declared: &[(&str, &str)],
    reads: &[Read],
) -> (Value, CheckedPackageEvidence, Vec<usize>) {
    let digest = sha256_hex(&canonical(document));
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": IDENTITY, "digest_domain": "sha256-jcs", "digest": digest,
    }]);
    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    let mut ordered: Vec<Value> = Vec::new();
    for (short, form) in declared {
        let key = declaration_key(short, form);
        let tag = if *form == "relationship" {
            "relation"
        } else {
            "model"
        };
        let mut node = wire_node(&key, tag, form, &key, &[], empty());
        if matches!(*form, OBJECT | INTERFACE | "relationship") {
            node["owner"] = json!({
                "kind": "model", "identity": IDENTITY, "node": orders(short),
            });
        }
        ordered.push(node);
    }
    let mut read_nodes = Vec::new();
    for read in reads {
        let ty = type_nodes(&read.ty, &mut nodes);
        let owner = declaration_key(read.owner, OBJECT);
        let body = json!({
            "term": "application",
            "operator": "query",
            "operation": {
                "identity": "quire.op.record.project", "laws": [], "mode": null,
                "member": {"kind": "field", "declaration": node_id(&owner), "name": read.field},
                "leaves": [],
            },
            "result_type": node_id(&ty),
            "arguments": [reference(&owner)],
        });
        let key = key_of(&json!({
            "version": APPLICATION_NODE,
            "node_tag": "expression",
            "semantic_form": "query",
            "semantic_type": node_id(&ty),
            "declaration": null,
            "recursion": null,
            "body": body,
        }));
        read_nodes.push(wire_node(&key, "expression", "query", &ty, &[&owner], body));
    }
    let mut graph: Vec<Value> = nodes.into_values().collect();
    graph.extend(ordered);
    let first_read = graph.len();
    graph.extend(read_nodes);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(graph);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let mut evidence = evidence_for(&package);
    evidence.insert_domain_package_document(digest, canonical(document));
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - first_read
        - reads.len();
    let positions = (0..reads.len())
        .map(|offset| base + first_read + offset)
        .collect();
    (package, evidence, positions)
}

fn read_with(
    package: &Value,
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(&canonical(package), limits, evidence)
}

fn admit(document: &Value, declared: &[(&str, &str)], reads: &[Read]) -> CheckedPackageV2 {
    let (package, evidence, _) = self::package(document, declared, reads);
    match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn declaration_id(short: &str) -> CheckedNodeId {
    serde_json::from_value(node_id(&declaration_key(short, OBJECT))).expect("a node id")
}

fn fields_of(package: &CheckedPackageV2, short: &str) -> CheckedModelObjectFields {
    package
        .model_object_fields(&declaration_id(short))
        .expect("the object's fields")
}

const ORDER: &str = "Order";

fn range(lower: i128, upper: i128) -> CheckedMemberType {
    CheckedMemberType::IntRange { lower, upper }
}

/// A small admitted graph for the public scalar operand API. The integer
/// bounds and application key use the same independently built preimages as
/// this module's model-field cases.
fn scalar_operand_package(arguments: Vec<Value>) -> (CheckedPackageV2, CheckedNodeId) {
    scalar_operand_package_for("quire.op.integer.add", "binary", arguments)
}

fn scalar_operand_package_for(
    operation: &str,
    operator: &str,
    arguments: Vec<Value>,
) -> (CheckedPackageV2, CheckedNodeId) {
    scalar_operand_package_with_bounds(operation, operator, arguments, Some(("0", "1000")))
}

fn scalar_operand_package_with_bounds(
    operation: &str,
    operator: &str,
    arguments: Vec<Value>,
    bounds: Option<(&str, &str)>,
) -> (CheckedPackageV2, CheckedNodeId) {
    scalar_operand_package_with_literal(operation, operator, arguments, bounds, "7")
}

fn scalar_operand_package_with_literal(
    operation: &str,
    operator: &str,
    mut arguments: Vec<Value>,
    bounds: Option<(&str, &str)>,
    literal_value: &str,
) -> (CheckedPackageV2, CheckedNodeId) {
    let mut nodes = BTreeMap::new();
    let integer = type_nodes(&Ty::Integer, &mut nodes);
    let bounded = if let Some((lower, upper)) = bounds {
        let body = bounds_body(&integer, lower, upper);
        let key = structural(
            "bounded_domain",
            "integer_range",
            Some(&integer),
            None,
            &body,
        );
        nodes.insert(
            key.clone(),
            wire_node(
                &key,
                "bounded_domain",
                "integer_range",
                &integer,
                &[&integer],
                body,
            ),
        );
        key
    } else {
        integer.clone()
    };
    let text = structural("scalar_type", "text", None, None, &empty());
    nodes.insert(
        text.clone(),
        wire_node(&text, "scalar_type", "text", &text, &[], empty()),
    );
    let mut parameter_keys = Vec::new();
    for name in ["left", "right"] {
        let body = json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value":
                {"term": "literal", "type": node_id(&text), "value_kind": "text", "value": name}},
            {"term": "binding", "name": "level", "value":
                {"term": "literal", "type": node_id(&integer), "value_kind": "integer", "value": "0"}},
        ]});
        let key = structural("value", "parameter", Some(&bounded), None, &body);
        let mut node = wire_node(&key, "value", "parameter", &bounded, &[], body);
        node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        nodes.insert(key.clone(), node);
        parameter_keys.push(key);
    }
    let literal_body = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": literal_value});
    let literal_key = structural("value", "literal", Some(&bounded), None, &literal_body);
    let mut literal_node = wire_node(
        &literal_key,
        "value",
        "literal",
        &bounded,
        &[&integer],
        literal_body,
    );
    literal_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    nodes.insert(literal_key.clone(), literal_node);
    for argument in &mut arguments {
        let Some(target) = argument.pointer_mut("/target/digest") else {
            continue;
        };
        let replacement = match target.as_str() {
            Some(key) if key == "a".repeat(64) => Some(&parameter_keys[0]),
            Some(key) if key == "b".repeat(64) => Some(&parameter_keys[1]),
            Some(key) if key == "c".repeat(64) => Some(&literal_key),
            _ => None,
        };
        if let Some(replacement) = replacement {
            *target = json!(replacement);
        }
    }
    if let Some(position) = arguments.iter().position(|argument| argument == "nested") {
        let inline = json!({"term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": "7"});
        let nested_body = json!({
            "term": "application", "operator": "binary",
            "operation": {"identity": "quire.op.integer.add", "laws": [],
                "mode": null, "member": null, "leaves": []},
            "result_type": node_id(&bounded), "arguments": [inline.clone(), inline],
        });
        let nested_key = key_of(&json!({
            "version": APPLICATION_NODE, "node_tag": "expression", "semantic_form": "binary",
            "semantic_type": node_id(&bounded), "declaration": null, "recursion": null,
            "body": nested_body,
        }));
        let mut nested = wire_node(
            &nested_key,
            "expression",
            "binary",
            &bounded,
            &[],
            nested_body,
        );
        nested["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        nodes.insert(nested_key.clone(), nested);
        arguments[position] = reference(&nested_key);
    }
    let body = json!({
        "term": "application", "operator": operator,
        "operation": {"identity": operation, "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(&bounded), "arguments": arguments,
    });
    let key = key_of(&json!({
        "version": APPLICATION_NODE, "node_tag": "expression", "semantic_form": operator,
        "semantic_type": node_id(&bounded), "declaration": null, "recursion": null,
        "body": body,
    }));
    let mut application = wire_node(&key, "expression", operator, &bounded, &[], body);
    application["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes.into_values());
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(application);
    crate::support::checked_package::settle(&mut package);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .last_mut()
        .expect("application")["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "expression", "ordinal": 1}));
    let mut second_source = package["source_map"]
        .as_array()
        .expect("source map")
        .last()
        .expect("application source")
        .clone();
    second_source["ordinal"] = json!(1);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second_source);
    refresh_identity(&mut package);
    let application_id: CheckedNodeId = serde_json::from_value(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .last()
            .expect("application")["node_id"]
            .clone(),
    )
    .expect("node id");
    let evidence = evidence_for(&package);
    let admitted = match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected scalar fixture admission, got {other:?}"),
    };
    (admitted, application_id)
}

/// The child's independently built QSL key: the public accessor test does
/// not inspect the admitted graph body to find its expected identity.
fn scalar_fixture_child_id(form: &str, value: &str) -> Value {
    let integer = structural("scalar_type", "integer", None, None, &empty());
    let bounds = bounds_body(&integer, "0", "1000");
    let bounded = structural(
        "bounded_domain",
        "integer_range",
        Some(&integer),
        None,
        &bounds,
    );
    let body = if form == "parameter" {
        let text = structural("scalar_type", "text", None, None, &empty());
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value":
                {"term": "literal", "type": node_id(&text), "value_kind": "text", "value": value}},
            {"term": "binding", "name": "level", "value":
                {"term": "literal", "type": node_id(&integer), "value_kind": "integer", "value": "0"}},
        ]})
    } else {
        assert_eq!(form, "literal");
        json!({"term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": value})
    };
    node_id(&structural("value", form, Some(&bounded), None, &body))
}

/// Tracing: TC-048, FR-038-AC-159, FR-038-AC-160, FR-038-AC-161, FR-038-AC-164
#[trace(
    "TC-048",
    "FR-038-AC-159",
    "FR-038-AC-160",
    "FR-038-AC-161",
    "FR-038-AC-164"
)]
#[test]
fn tc_048_scalar_operands_keep_argument_identity_and_exact_range() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperand, CheckedScalarOperandChild,
        CheckedScalarOperandError,
    };
    fn consume_child(entry: &CheckedScalarOperand) -> (u64, i128, i128, CheckedNodeId) {
        let node = match &entry.child {
            CheckedScalarOperandChild::GraphChild(node) => node.clone(),
            CheckedScalarOperandChild::InlineLiteral {
                application,
                occurrence,
                ordinal,
            } => {
                assert_eq!(*ordinal, entry.ordinal);
                assert_eq!(occurrence.role, CheckedOccurrenceRole::Expression);
                application.clone()
            }
        };
        (entry.ordinal, entry.range.lower, entry.range.upper, node)
    }
    fn consume_error(error: CheckedScalarOperandError) -> &'static str {
        match error {
            CheckedScalarOperandError::UnknownNode => "unknown node",
            CheckedScalarOperandError::NotApplication => "not application",
            CheckedScalarOperandError::MissingOccurrence => "missing occurrence",
            CheckedScalarOperandError::UnknownOperator => "unknown operator",
            CheckedScalarOperandError::IneligibleOperator => "ineligible operator",
            CheckedScalarOperandError::MissingChild => "missing child",
            CheckedScalarOperandError::MissingRange => "missing range",
            CheckedScalarOperandError::UnboundedRange => "unbounded range",
            CheckedScalarOperandError::RangeOutOfI128 => "out of i128",
        }
    }
    let first = node_id(&"a".repeat(64));
    let integer = structural("scalar_type", "integer", None, None, &empty());
    let inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    let (package, application) =
        scalar_operand_package(vec![json!({"term": "reference", "target": first}), inline]);
    let first = scalar_fixture_child_id("parameter", "left");
    let second = scalar_fixture_child_id("parameter", "right");
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let before = package.clone();
    let entries = package
        .scalar_application_operands(&application, &occurrence)
        .expect("typed operands");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].ordinal, 0);
    assert_eq!(entries[0].range.lower, 0);
    assert_eq!(entries[0].range.upper, 1000);
    assert_eq!(entries[1].ordinal, 1);
    assert_eq!(consume_child(&entries[1]).0, 1);
    assert_eq!((entries[1].range.lower, entries[1].range.upper), (7, 7));
    assert!(matches!(
        &entries[0].child,
        CheckedScalarOperandChild::GraphChild(id) if *id == serde_json::from_value(first).expect("id")
    ));
    assert!(matches!(
        &entries[1].child,
        CheckedScalarOperandChild::InlineLiteral { application: id, occurrence: at, ordinal: 1 }
            if id == &application && at == &occurrence
    ));
    assert_eq!(
        entries,
        package
            .clone()
            .scalar_application_operands(&application, &occurrence)
            .expect("repeat")
    );
    assert_eq!(package, before);
    assert_eq!(
        consume_error(
            package
                .scalar_application_operands(
                    &application,
                    &CheckedOccurrence {
                        role: CheckedOccurrenceRole::Expression,
                        ordinal: 2,
                    },
                )
                .expect_err("no third occurrence"),
        ),
        "missing occurrence"
    );

    let second_occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 1,
    };
    let second_entries = package
        .scalar_application_operands(&application, &second_occurrence)
        .expect("second occurrence");
    assert!(matches!(&second_entries[1].child,
        CheckedScalarOperandChild::InlineLiteral { application: id, occurrence: at, ordinal: 1 }
        if id == &application && at == &second_occurrence));
    assert_ne!(second_entries[1].child, entries[1].child);

    let (swapped, swapped_id) = scalar_operand_package(vec![
        json!({"term": "reference", "target": second}),
        json!({"term": "reference", "target": node_id(&"a".repeat(64))}),
    ]);
    let result = swapped
        .scalar_application_operands(&swapped_id, &occurrence)
        .expect("swapped");
    assert_eq!(result.len(), 2);
    assert!(
        matches!(&result[0].child, CheckedScalarOperandChild::GraphChild(id)
        if *id == serde_json::from_value(second).expect("id"))
    );
    assert_ne!(result[0].child, result[1].child);
    let (forward, forward_id) = scalar_operand_package(vec![
        json!({"term": "reference", "target": node_id(&"a".repeat(64))}),
        json!({"term": "reference", "target": node_id(&"b".repeat(64))}),
    ]);
    let forward = forward
        .scalar_application_operands(&forward_id, &occurrence)
        .expect("forward");
    assert_eq!(forward[0].child, result[1].child);
    assert_eq!(forward[1].child, result[0].child);

    let graph_literal = node_id(&"c".repeat(64));
    let (package, id) = scalar_operand_package(vec![
        json!({"term": "reference", "target": graph_literal.clone()}),
        json!({"term": "reference", "target": node_id(&"a".repeat(64))}),
    ]);
    let graph_literal = scalar_fixture_child_id("literal", "7");
    let entries = package
        .scalar_application_operands(&id, &occurrence)
        .expect("literal child");
    assert_eq!((entries[0].range.lower, entries[0].range.upper), (7, 7));
    assert!(
        matches!(&entries[0].child, CheckedScalarOperandChild::GraphChild(child)
        if *child == serde_json::from_value(graph_literal).expect("id"))
    );

    let (package, outer) = scalar_operand_package(vec![
        json!("nested"),
        json!({"term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": "7"}),
    ]);
    let inner = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "binary" && node.node_id != outer)
        .expect("inner application");
    let entries = package
        .scalar_application_operands(&outer, &occurrence)
        .expect("nested child");
    assert!(
        matches!(&entries[0].child, CheckedScalarOperandChild::GraphChild(id)
        if *id == inner.node_id)
    );
    assert_eq!((entries[0].range.lower, entries[0].range.upper), (0, 1000));
}

/// Tracing: TC-048, FR-038-AC-161, FR-038-AC-162
#[trace("TC-048", "FR-038-AC-161", "FR-038-AC-162")]
#[test]
fn tc_048_scalar_operands_refuse_unknown_node_nonapplication_and_missing_occurrence() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperandError,
    };
    let integer = structural("scalar_type", "integer", None, None, &empty());
    let inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    let (package, application) = scalar_operand_package(vec![inline.clone(), inline]);
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let missing = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 2,
    };
    let operands = package
        .scalar_application_operands(&application, &occurrence)
        .expect("two inline terms");
    assert_ne!(operands[0].child, operands[1].child);
    assert_eq!(
        package.scalar_application_operands(&application, &missing),
        Err(CheckedScalarOperandError::MissingOccurrence)
    );
    assert_eq!(
        package.scalar_application_operands(
            &serde_json::from_value(node_id(&integer)).expect("id"),
            &occurrence
        ),
        Err(CheckedScalarOperandError::NotApplication)
    );
    assert_eq!(
        package.scalar_application_operands(
            &serde_json::from_value(node_id(&"f".repeat(64))).expect("id"),
            &occurrence
        ),
        Err(CheckedScalarOperandError::UnknownNode)
    );
}

/// Tracing: TC-048, FR-038-AC-159, FR-038-AC-163
#[trace("TC-048", "FR-038-AC-159", "FR-038-AC-163")]
#[test]
fn tc_048_scalar_operand_eligibility_uses_catalogued_integer_operations() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperandError,
    };
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let integer = structural("scalar_type", "integer", None, None, &empty());
    let literal = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    for (identity, operator, count) in [
        ("quire.op.integer.add", "binary", 2),
        ("quire.op.integer.sub", "binary", 2),
        ("quire.op.integer.mul", "binary", 2),
        ("quire.op.integer.negate", "unary", 1),
    ] {
        let (package, node) =
            scalar_operand_package_for(identity, operator, vec![literal.clone(); count]);
        let entries = package
            .scalar_application_operands(&node, &occurrence)
            .expect(identity);
        assert_eq!(entries.len(), count, "{identity}");
        assert!(entries.iter().enumerate().all(|(position, entry)| {
            entry.ordinal == u64::try_from(position).expect("small fixture")
                && entry.range.lower == 7
                && entry.range.upper == 7
        }));
    }
    let (package, node) = scalar_operand_package_for(
        "quire.op.integer.mod",
        "binary",
        vec![literal.clone(), literal],
    );
    assert_eq!(
        package.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::IneligibleOperator)
    );
}

/// Tracing: TC-048, FR-038-AC-162
#[trace("TC-048", "FR-038-AC-162")]
#[test]
fn tc_048_scalar_operands_refuse_unbounded_and_out_of_i128_ranges() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperandError,
    };
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let first = json!({"term": "reference", "target": node_id(&"a".repeat(64))});
    let integer = structural("scalar_type", "integer", None, None, &empty());
    let inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    let (unbounded, node) = scalar_operand_package_with_bounds(
        "quire.op.integer.add",
        "binary",
        vec![first.clone(), inline.clone()],
        None,
    );
    assert_eq!(
        unbounded.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::UnboundedRange)
    );

    for (lower, upper) in [
        ("0", "170141183460469231731687303715884105728"),
        ("-170141183460469231731687303715884105729", "0"),
    ] {
        let (package, node) = scalar_operand_package_with_bounds(
            "quire.op.integer.add",
            "binary",
            vec![first.clone(), inline.clone()],
            Some((lower, upper)),
        );
        assert_eq!(
            package.scalar_application_operands(&node, &occurrence),
            Err(CheckedScalarOperandError::RangeOutOfI128),
            "{lower}..={upper}"
        );
    }

    let beyond = "170141183460469231731687303715884105728";
    let large_inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": beyond});
    let (package, node) = scalar_operand_package(vec![large_inline, inline]);
    assert_eq!(
        package.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::RangeOutOfI128)
    );

    let (package, node) = scalar_operand_package_with_literal(
        "quire.op.integer.add",
        "binary",
        vec![
            json!({"term": "reference", "target": node_id(&"c".repeat(64))}),
            json!({"term": "literal", "type": node_id(&integer),
                "value_kind": "integer", "value": "7"}),
        ],
        None,
        beyond,
    );
    assert_eq!(
        package.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::RangeOutOfI128)
    );

    let minimum = i128::MIN.to_string();
    let maximum = i128::MAX.to_string();
    let (package, node) = scalar_operand_package_with_bounds(
        "quire.op.integer.add",
        "binary",
        vec![
            first,
            json!({"term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": "7"}),
        ],
        Some((&minimum, &maximum)),
    );
    let entries = package
        .scalar_application_operands(&node, &occurrence)
        .expect("i128 edges");
    assert_eq!(
        (entries[0].range.lower, entries[0].range.upper),
        (i128::MIN, i128::MAX)
    );
}

fn names(fields: &CheckedModelObjectFields) -> Vec<&str> {
    fields.fields().iter().map(|field| field.name()).collect()
}

// --- AC-136 -----------------------------------------------------------------------

/// A domain document declaring `balance` and `audit`, each `Int[0, 1000]`,
/// and a read of `balance` only: both fields come back, ascending by name,
/// with `i128` bounds, although no read names `audit`; an absent field is
/// `None`, not an error.
///
/// Trace: FR-038-AC-136
#[trace("TC-227", "FR-038-AC-136")]
#[test]
fn tc_227_the_fields_of_a_model_object_come_back_as_values() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(
            ORDER,
            &[],
            &[field("balance", &thousand), field("audit", &thousand)],
        ),
    ]);
    let read = Read {
        owner: ORDER,
        field: "balance",
        ty: Ty::Range(0, 1000),
    };
    let package = admit(&document, &[(ORDER, OBJECT)], &[read]);
    let fields = fields_of(&package, ORDER);
    assert_eq!(names(&fields), ["audit", "balance"]);
    for field in fields.fields() {
        assert_eq!(field.member_type(), Some(&range(0, 1000)));
    }
    let audit = fields
        .field("audit")
        .expect("audit is a field no read names");
    assert_eq!(audit.name(), "audit");
    assert!(fields.field("missing").is_none());
    // The order is the name's, not the document's: `zeta` is declared first.
    let document = self::document(vec![
        thousand_type(),
        object(
            ORDER,
            &[],
            &[field("zeta", &thousand), field("alpha", &thousand)],
        ),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    assert_eq!(names(&fields_of(&package, ORDER)), ["alpha", "zeta"]);
}

fn thousand_type() -> Value {
    thousand()
}

// --- AC-137 and AC-141 -----------------------------------------------------------

/// The package of `document` with `Order`'s field `name` read at `ty`:
/// whether it admits, and when refused the refusal's code, cause and path.
fn read_of(
    document: &Value,
    declared: &[(&str, &str)],
    owner: &'static str,
    name: &'static str,
    ty: Ty,
) -> Result<
    (),
    (
        CheckedPackageRefusalCode,
        Option<CheckedPackageRefusalCause>,
        String,
    ),
> {
    let read = Read {
        owner,
        field: name,
        ty,
    };
    let (package, evidence, positions) = package(document, declared, &[read]);
    match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(_) => Ok(()),
        CheckedPackageV2ReadResult::Refused(refusal) => {
            let at = positions.first().copied().expect("a read position");
            let result_type = format!("/semantic_graph/nodes/{at}/body/result_type");
            let path = refusal.path.as_ref().map(ToString::to_string);
            assert_eq!(path.as_deref(), Some(result_type.as_str()), "{refusal:?}");
            Err((refusal.code, refusal.cause, result_type))
        }
        other => panic!("expected admission or a refusal, got {other:?}"),
    }
}

const INELIGIBLE: (
    CheckedPackageRefusalCode,
    Option<CheckedPackageRefusalCause>,
) = (
    CheckedPackageRefusalCode::IllTyped,
    Some(CheckedPackageRefusalCause::OperatorIneligible),
);

/// A chain with an inherited field, a field redefined by a subtype with a
/// narrower range, a most-derived redefiner two owners deep and an own field:
/// each exposed name comes back once, with the most-derived redefiner's type
/// and no hidden field; a read keyed from the returned type admits and the
/// same read keyed from the hidden base field's type refuses.
///
/// Trace: FR-038-AC-137
#[trace("TC-227", "FR-038-AC-137")]
#[test]
fn tc_227_the_effective_set_is_the_one_admission_resolves() {
    let (hundred, ten) = (orders("Hundred"), orders("Ten"));
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        value_type("Hundred", "integer", Some(("0", "100"))),
        value_type("Ten", "integer", Some(("0", "10"))),
        object(
            "Base",
            &[],
            &[field("limit", &thousand), field("inherited", &thousand)],
        ),
        object(
            "Mid",
            &["Base"],
            &[field("limit", &hundred).redefining(&format!("{}/limit", orders("Base")))],
        ),
        object(
            ORDER,
            &["Mid"],
            &[
                field("limit", &ten).redefining(&format!("{}/limit", orders("Mid"))),
                field("own", &thousand),
            ],
        ),
    ]);
    let declared = [(ORDER, OBJECT), ("Base", OBJECT), ("Mid", OBJECT)];
    let package = admit(&document, &declared, &[]);
    let fields = fields_of(&package, ORDER);
    assert_eq!(names(&fields), ["inherited", "limit", "own"]);
    assert_eq!(
        fields.field("limit").and_then(|f| f.member_type()),
        Some(&range(0, 10))
    );
    assert_eq!(
        fields.field("inherited").and_then(|f| f.member_type()),
        Some(&range(0, 1000))
    );
    for field in fields.fields() {
        let name = match field.name() {
            "inherited" => "inherited",
            "limit" => "limit",
            _ => "own",
        };
        let returned = ty_of(field.member_type().expect("a typed field"));
        assert_eq!(
            read_of(&document, &declared, ORDER, name, returned),
            Ok(()),
            "a read of {name} keyed from the returned type admits"
        );
    }
    // The hidden base field's type is not the field's type.
    for hidden in [Ty::Range(0, 1000), Ty::Range(0, 100)] {
        let refused = read_of(&document, &declared, ORDER, "limit", hidden)
            .expect_err("a read keyed from a hidden field's type refuses");
        assert_eq!((refused.0, refused.1), INELIGIBLE);
    }
    // The less derived owners expose their own redefinition.
    assert_eq!(
        fields_of(&package, "Mid")
            .field("limit")
            .and_then(|f| f.member_type()),
        Some(&range(0, 100))
    );
    assert_eq!(
        fields_of(&package, "Base")
            .field("limit")
            .and_then(|f| f.member_type()),
        Some(&range(0, 1000))
    );
}

// --- AC-138 and AC-141 -----------------------------------------------------------

const MAX: &str = "170141183460469231731687303715884105727";
const MIN: &str = "-170141183460469231731687303715884105728";
const PAST_MAX: &str = "170141183460469231731687303715884105728";
const PAST_MIN: &str = "-170141183460469231731687303715884105729";

/// The document of AC-138: a field of every member kind the tables derive and
/// of every kind they do not.
fn every_kind() -> Value {
    let thousand = orders("Thousand");
    let many = |ordered, unique| (0, None, ordered, unique);
    document(vec![
        thousand_type(),
        value_type("Zero", "integer", Some(("0", "0"))),
        value_type("Signed", "integer", Some(("-5", "5"))),
        value_type("Wide", "integer", Some((MIN, MAX))),
        // 2^53 + 1: a bound an `f64` cannot hold.
        value_type("Odd", "integer", Some(("0", "9007199254740993"))),
        value_type("PastMax", "integer", Some(("0", PAST_MAX))),
        value_type("PastMin", "integer", Some((PAST_MIN, "0"))),
        value_type("Label", "text", None),
        typed_object("Pump", "interface", &[], &[]),
        typed_object("Alarm", "event", &[], &[]),
        object("Other", &[], &[]),
        object(
            ORDER,
            &[],
            &[
                field("flag", &format!("{NATIVE}Boolean")),
                field("number", &format!("{NATIVE}Integer")),
                field("zero", &orders("Zero")),
                field("signed", &orders("Signed")),
                field("wide", &orders("Wide")),
                field("odd", &orders("Odd")),
                field("link", &orders("Other")),
                field("maybe", &thousand).optional(),
                field("set", &thousand).many(many(false, true)),
                field("bag", &thousand).many(many(false, false)),
                field("sequence", &thousand).many(many(true, false)),
                field("ordered", &thousand).many(many(true, true)),
                field("window", &thousand).many((1, Some(4), false, true)),
                field("maybe_set", &thousand)
                    .optional()
                    .many(many(false, true)),
                field("text", &format!("{NATIVE}Text")),
                field("rational", &format!("{NATIVE}Rational")),
                field("decimal", &format!("{NATIVE}Decimal")),
                field("float", &format!("{NATIVE}Float32")),
                field("pump", &orders("Pump")),
                field("alarm", &orders("Alarm")),
                field("label", &orders("Label")),
                field("past_max", &orders("PastMax")),
                field("past_min", &orders("PastMin")),
                field("at_least_one", &thousand).many((1, None, false, true)),
            ],
        ),
    ])
}

/// What AC-138 says each field returns.
fn expected_kinds() -> Vec<(&'static str, Option<Ty>)> {
    let thousand = || Ty::Range(0, 1000);
    let collection = |kind, bounds| Some(Ty::Collection(kind, Box::new(thousand()), bounds));
    let other = declaration_key("Other", OBJECT);
    vec![
        ("flag", Some(Ty::Boolean)),
        ("number", Some(Ty::Integer)),
        ("zero", Some(Ty::Range(0, 0))),
        ("signed", Some(Ty::Range(-5, 5))),
        ("wide", Some(Ty::Range(i128::MIN, i128::MAX))),
        ("odd", Some(Ty::Range(0, 9_007_199_254_740_993))),
        ("link", Some(Ty::Reference(other))),
        ("maybe", Some(Ty::Option(Box::new(thousand())))),
        ("set", collection(Kind::Set, None)),
        ("bag", collection(Kind::Bag, None)),
        ("sequence", collection(Kind::Sequence, None)),
        ("ordered", collection(Kind::OrderedSet, None)),
        ("window", collection(Kind::Set, Some((1, 4)))),
        (
            "maybe_set",
            Some(Ty::Option(Box::new(
                collection(Kind::Set, None).expect("a collection"),
            ))),
        ),
        ("text", None),
        ("rational", None),
        ("decimal", None),
        ("float", None),
        ("pump", None),
        ("alarm", None),
        ("label", None),
        ("past_max", None),
        ("past_min", None),
        ("at_least_one", None),
    ]
}

fn kinds_package() -> CheckedPackageV2 {
    admit(
        &every_kind(),
        &[(ORDER, OBJECT), ("Other", OBJECT), ("Pump", INTERFACE)],
        &[],
    )
}

/// Each member kind returns the exact value the tables derive, the `i128`
/// extremes without loss and one past each as `None`, and a field with no type
/// is present beside the others and never refuses the call.
///
/// Trace: FR-038-AC-138
#[trace("TC-227", "FR-038-AC-138")]
#[test]
fn tc_227_every_member_kind_returns_its_derived_type_or_none() {
    let package = kinds_package();
    let fields = fields_of(&package, ORDER);
    let mut expected = expected_kinds();
    expected.sort_by_key(|(name, _)| *name);
    assert_eq!(fields.fields().len(), expected.len());
    for ((name, ty), field) in expected.iter().zip(fields.fields()) {
        assert_eq!(field.name(), *name);
        assert_eq!(
            field.member_type().map(ty_of),
            *ty,
            "the member type of {name}"
        );
    }
    // The extremes are values, not floats or 64-bit integers.
    assert_eq!(
        fields.field("wide").and_then(|f| f.member_type()),
        Some(&range(i128::MIN, i128::MAX))
    );
    assert!(matches!(
        fields.field("link").and_then(|f| f.member_type()),
        Some(CheckedMemberType::Reference(target)) if *target == declaration_id("Other")
    ));
}

/// The perturbation of AC-141: a type that stays well formed and is not the
/// field's.
fn perturbed(ty: &Ty, other: &str) -> Ty {
    match ty {
        Ty::Boolean => Ty::Integer,
        Ty::Integer => Ty::Boolean,
        Ty::Range(lower, upper) if upper > lower => Ty::Range(*lower, upper - 1),
        Ty::Range(lower, upper) if *lower < i128::MAX => Ty::Range(*lower, upper + 1),
        Ty::Range(lower, upper) => Ty::Range(lower - 1, *upper),
        Ty::Reference(_) => Ty::Reference(other.to_owned()),
        Ty::Option(element) => Ty::Option(Box::new(perturbed(element, other))),
        Ty::Collection(kind, element, Some((lower, upper))) => {
            let bounds = if upper > lower {
                (*lower, upper - 1)
            } else if *lower < u64::MAX {
                (*lower, upper + 1)
            } else {
                (lower - 1, *upper)
            };
            Ty::Collection(*kind, element.clone(), Some(bounds))
        }
        Ty::Collection(kind, element, None) => {
            Ty::Collection(*kind, Box::new(perturbed(element, other)), None)
        }
    }
}

/// For each field with a derived type, a read keyed by the key recomputed here
/// from the returned type admits, and the same read keyed from the perturbed
/// type refuses `ill_typed`/`operator-ineligible` at the read: the accessor
/// reports the type admission compares.
///
/// Trace: FR-038-AC-141
#[trace("TC-227", "FR-038-AC-141")]
#[test]
fn tc_227_the_returned_type_is_the_type_admission_compares() {
    let package = kinds_package();
    let fields = fields_of(&package, ORDER);
    let document = every_kind();
    let declared = [(ORDER, OBJECT), ("Other", OBJECT), ("Pump", INTERFACE)];
    let other = declaration_key(ORDER, OBJECT);
    let mut checked = 0;
    for (name, _) in expected_kinds() {
        let Some(returned) = fields
            .field(name)
            .and_then(|field| field.member_type())
            .map(ty_of)
        else {
            continue;
        };
        let name = expected_kinds()
            .iter()
            .map(|(candidate, _)| *candidate)
            .find(|candidate| *candidate == name)
            .expect("a declared field");
        assert_eq!(
            read_of(&document, &declared, ORDER, name, returned.clone()),
            Ok(()),
            "a read of {name} keyed from {returned:?}"
        );
        let perturbed = perturbed(&returned, &other);
        assert_ne!(perturbed, returned);
        let refused = read_of(&document, &declared, ORDER, name, perturbed.clone())
            .expect_err("a read keyed from the perturbed type refuses");
        assert_eq!((refused.0, refused.1), INELIGIBLE, "{name}: {perturbed:?}");
        checked += 1;
    }
    assert_eq!(checked, 14, "every field with a derived type was exercised");
}

// --- AC-139 ----------------------------------------------------------------------

/// The package of AC-139: `Order` and `Holder` (an object type with an
/// unread declaration node), a systems interface, and the scalar node a read
/// would name.
fn refusal_document() -> Value {
    let thousand = orders("Thousand");
    document(vec![
        thousand_type(),
        typed_object("Pump", "interface", &[], &[]),
        object(ORDER, &[], &[field("balance", &thousand)]),
    ])
}

/// `UnknownNode` and `NotModelObjectType` are two errors: an id no node has,
/// a scalar node, a systems interface's declaration node and a node keyed as
/// no selected declaration each name what is wrong, and the call is `Ok` for
/// an absent field name.
///
/// Trace: FR-038-AC-139, FR-038-AC-155
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-155")]
#[test]
fn tc_227_the_accessor_tells_an_unknown_node_from_a_node_that_is_no_object_type() {
    let read = Read {
        owner: ORDER,
        field: "balance",
        ty: Ty::Range(0, 1000),
    };
    let package = admit(
        &refusal_document(),
        &[(ORDER, OBJECT), ("Pump", INTERFACE)],
        &[read],
    );
    let id = |digest: &str| -> CheckedNodeId {
        serde_json::from_value(node_id(digest)).expect("a node id")
    };
    assert_eq!(
        package.model_object_fields(&id(&"9".repeat(64))),
        Err(CheckedModelFieldsError::UnknownNode)
    );
    // The scalar node of the read's type.
    let mut scalar = BTreeMap::new();
    let integer = type_nodes(&Ty::Integer, &mut scalar);
    let thousand = type_nodes(&Ty::Range(0, 1000), &mut scalar);
    for key in [&integer, &thousand] {
        assert_eq!(
            package.model_object_fields(&id(key)),
            Err(CheckedModelFieldsError::NotModelObjectType),
            "a type node is no model object"
        );
    }
    assert_eq!(
        package.model_object_fields(&id(&declaration_key("Pump", INTERFACE))),
        Err(CheckedModelFieldsError::NotModelObjectType),
        "a systems interface is no object type"
    );
    let fields = fields_of(&package, ORDER);
    assert!(fields.field("absent").is_none());
    assert_eq!(names(&fields), ["balance"]);
}

/// A model node the document does not select, and one whose body is not
/// `aggregate{[]}`, are admitted when unread and are `NotModelObjectType`; two
/// packages that differ only in that node's body return `Ok` and
/// `NotModelObjectType`.
///
/// Trace: FR-038-AC-139
#[trace("TC-227", "FR-038-AC-139")]
#[test]
fn tc_227_a_model_node_that_step_two_would_refuse_is_no_object_type() {
    let declared = [(ORDER, OBJECT), ("Pump", INTERFACE)];
    let (package, evidence, _) = package(&refusal_document(), &declared, &[]);
    let order = declaration_key(ORDER, OBJECT);
    let admitted = |package: &Value| match read_with(
        package,
        &evidence,
        CheckedPackageReadLimits::bounded(),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    };
    let intact = admitted(&package);
    assert!(intact.model_object_fields(&declaration_id(ORDER)).is_ok());
    // The same package, with the declaration node's body not empty.
    let mut tampered = package.clone();
    for node in tampered["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_id"]["digest"] == json!(order) {
            node["body"] = json!({"term": "aggregate", "members": [
                reference(&declaration_key("Pump", INTERFACE)),
            ]});
            node["dependencies"] = json!([node_id(&declaration_key("Pump", INTERFACE))]);
        }
    }
    refresh_identity(&mut tampered);
    let tampered = admitted(&tampered);
    assert_eq!(
        tampered.model_object_fields(&declaration_id(ORDER)),
        Err(CheckedModelFieldsError::NotModelObjectType),
        "an unread node with a body is admitted and is no model object type"
    );
    assert_ne!(intact, tampered, "the packages differ in that body alone");
    // A node keyed as no selected declaration.
    let mut unselected = package;
    let stranger = "a".repeat(64);
    let mut stranger_node = wire_node(&stranger, "model", OBJECT, &stranger, &[], empty());
    stranger_node["owner"] = json!({
        "kind": "model", "identity": "acme/other", "node": orders("Stranger"),
    });
    unselected["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(stranger_node);
    rebuild_source_map(&mut unselected);
    refresh_identity(&mut unselected);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&unselected, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("an unselected model owner must refuse admission");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
}

/// An object type whose document gives two unhidden effective fields of one
/// name admits when no read names it and returns the error naming the name,
/// for the whole call although the object also declares a valid field; the
/// same document with a read of the name is refused at admission.
///
/// Trace: FR-038-AC-139
#[trace("TC-227", "FR-038-AC-139")]
#[test]
fn tc_227_an_ambiguous_name_refuses_the_call_not_the_admission() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object("Left", &[], &[field("x", &thousand)]),
        object(
            ORDER,
            &["Left"],
            &[field("x", &thousand), field("y", &thousand)],
        ),
    ]);
    let declared = [(ORDER, OBJECT), ("Left", OBJECT)];
    let package = admit(&document, &declared, &[]);
    assert_eq!(
        package.model_object_fields(&declaration_id(ORDER)),
        Err(CheckedModelFieldsError::AmbiguousField("x".into()))
    );
    // The supertype alone is not ambiguous.
    assert_eq!(names(&fields_of(&package, "Left")), ["x"]);
    // Package B: the same document with a read of `x` is refused.
    let read = Read {
        owner: ORDER,
        field: "x",
        ty: Ty::Range(0, 1000),
    };
    let (package_b, evidence, _) = self::package(&document, &declared, &[read]);
    match read_with(&package_b, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::AmbiguousDeclaration
            );
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::AmbiguousName)
            );
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

// --- AC-142 ----------------------------------------------------------------------

/// The accessor takes `&self` and one node id and no evidence; the three
/// enums match exhaustively with no wildcard arm from outside the crate,
/// which compiles only while they are closed.
///
/// Trace: FR-038-AC-142
#[trace("TC-227", "FR-038-AC-142")]
#[test]
fn tc_227_the_public_shape_is_closed_and_takes_one_node_id() {
    let accessor: fn(
        &CheckedPackageV2,
        &CheckedNodeId,
    ) -> Result<CheckedModelObjectFields, CheckedModelFieldsError> =
        CheckedPackageV2::model_object_fields;
    let _ = accessor;
    fn member(ty: &CheckedMemberType) -> u8 {
        match ty {
            CheckedMemberType::Boolean => 0,
            CheckedMemberType::Integer => 1,
            CheckedMemberType::IntRange { .. } => 2,
            CheckedMemberType::Reference(_) => 3,
            CheckedMemberType::Option(_) => 4,
            CheckedMemberType::Collection { .. } => 5,
        }
    }
    fn collection(kind: CheckedCollectionKind) -> u8 {
        match kind {
            CheckedCollectionKind::Set => 0,
            CheckedCollectionKind::Bag => 1,
            CheckedCollectionKind::Sequence => 2,
            CheckedCollectionKind::OrderedSet => 3,
        }
    }
    fn error(error: &CheckedModelFieldsError) -> u8 {
        match error {
            CheckedModelFieldsError::UnknownNode => 0,
            CheckedModelFieldsError::NotModelObjectType => 1,
            CheckedModelFieldsError::AmbiguousField(_) => 2,
        }
    }
    assert_eq!(member(&CheckedMemberType::Boolean), 0);
    assert_eq!(collection(CheckedCollectionKind::OrderedSet), 3);
    assert_eq!(error(&CheckedModelFieldsError::UnknownNode), 0);
}

// --- AC-143 ----------------------------------------------------------------------

/// Two calls, a clone and two admissions of one package under different byte
/// limits agree, and the retained tables take no part in the package's
/// equality; a chain of 200 object types and 100 redefinitions of one field
/// is read without panic or stack overflow.
///
/// Trace: FR-038-AC-143
#[trace("TC-227", "FR-038-AC-143")]
#[test]
fn tc_227_the_accessor_is_pure_and_total() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(ORDER, &[], &[field("balance", &thousand)]),
    ]);
    let (package, evidence, _) = package(&document, &[(ORDER, OBJECT)], &[]);
    let admit_under = |bytes: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.bytes = bytes;
        match read_with(&package, &evidence, limits) {
            CheckedPackageV2ReadResult::Admitted(package) => *package,
            other => panic!("expected admission under {bytes} bytes, got {other:?}"),
        }
    };
    let roomy = CheckedPackageReadLimits::bounded().bytes;
    let (a, b) = (admit_under(roomy), admit_under(roomy / 2));
    assert_eq!(a, b, "packages admitted from one set of bytes are equal");
    let first = fields_of(&a, ORDER);
    assert_eq!(first, fields_of(&a, ORDER), "two calls agree");
    assert_eq!(first, fields_of(&a.clone(), ORDER), "a clone agrees");
    assert_eq!(
        first,
        fields_of(&b, ORDER),
        "a package read under other limits agrees"
    );

    // 200 types: `R1` declares `f`, `R2` to `R101` redefine their predecessor's
    // (100 redefinitions), `R102` to `R200` each declare a field of their own.
    let names: Vec<String> = (1..=200).map(|k| format!("R{k}")).collect();
    let own: Vec<String> = (102..=200).map(|k| format!("g{k}")).collect();
    let types: Vec<Value> = std::iter::once(thousand_type())
        .chain((1..=200).map(|k| {
            let supertypes: Vec<&str> = if k == 1 {
                vec![]
            } else {
                vec![names[k - 2].as_str()]
            };
            let declared = if k <= 101 {
                let mut declared = field("f", &thousand);
                if k > 1 {
                    declared = declared.redefining(&format!("{}/f", orders(&names[k - 2])));
                }
                declared
            } else {
                field(leak(&own[k - 102]), &thousand)
            };
            object(&names[k - 1], &supertypes, &[declared])
        }))
        .collect();
    let deep = admit(&document_of(types), &[("R200", OBJECT)], &[]);
    let fields = deep
        .model_object_fields(&declaration_id("R200"))
        .expect("the chain's fields");
    assert_eq!(fields.fields().len(), 100);
    assert_eq!(
        fields.field("f").and_then(|f| f.member_type()),
        Some(&range(0, 1000))
    );
}

fn document_of(types: Vec<Value>) -> Value {
    document(types)
}

/// A field name the document builder keeps for the program's life.
fn leak(text: &str) -> &'static str {
    Box::leak(text.to_owned().into_boxed_str())
}

// --- AC-144 ----------------------------------------------------------------------

fn chain(count: usize) -> Value {
    let thousand = orders("Thousand");
    let names: Vec<String> = (1..=count).map(|k| format!("C{k}")).collect();
    let fields: Vec<String> = (1..=count).map(|k| format!("f{k}")).collect();
    let types: Vec<Value> = std::iter::once(thousand_type())
        .chain((1..=count).map(|k| {
            let supertypes: Vec<&str> = if k == 1 {
                vec![]
            } else {
                vec![names[k - 2].as_str()]
            };
            object(
                &names[k - 1],
                &supertypes,
                &[field(leak(&fields[k - 1]), &thousand)],
            )
        }))
        .collect();
    document(types)
}

fn last(count: usize) -> String {
    format!("C{count}")
}

fn chain_package(count: usize) -> (Value, CheckedPackageEvidence) {
    let declared = last(count);
    let (package, evidence, _) = package(&chain(count), &[(declared.as_str(), OBJECT)], &[]);
    (package, evidence)
}

fn admits_under(package: &Value, evidence: &CheckedPackageEvidence, work: u64) -> bool {
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    match read_with(package, evidence, limits) {
        CheckedPackageV2ReadResult::Admitted(_) => true,
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                incomplete.path.map(|path| path.to_string()).as_deref(),
                Some("/lock/model_selections/0")
            );
            false
        }
        other => panic!("expected admission or `incomplete`, got {other:?}"),
    }
}

/// The smallest `work` limit under which `package` admits.
fn smallest_admitting_work(package: &Value, evidence: &CheckedPackageEvidence) -> u64 {
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(admits_under(package, evidence, high));
    while low < high {
        let middle = low + (high - low) / 2;
        if admits_under(package, evidence, middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// A chain of 4 types and a chain of 8 admit under the smallest `work` limit
/// `M` that admits them, and under `M - 1` are `incomplete` at
/// `/lock/model_selections/0`, though nothing reads them.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_chain_admits_at_its_smallest_limit_and_is_incomplete_one_below() {
    for count in [4, 8] {
        let (package, evidence) = chain_package(count);
        let smallest = smallest_admitting_work(&package, &evidence);
        assert!(smallest > 0);
        assert!(admits_under(&package, &evidence, smallest));
        assert!(!admits_under(&package, &evidence, smallest - 1));
    }
}

/// Under `CheckedPackageReadLimits::bounded()` a chain of 1000 types admits
/// and a chain of 1500 is `incomplete` at the selection's row: the owner
/// decision's numbers.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_the_long_chains_are_decided_by_the_bounded_work_limit() {
    let bounded = CheckedPackageReadLimits::bounded().work;
    assert_eq!(bounded, 1_000_000);
    let (package, evidence) = chain_package(1000);
    assert!(admits_under(&package, &evidence, bounded));
    let (package, evidence) = chain_package(1500);
    assert!(!admits_under(&package, &evidence, bounded));
}

/// The package of a chain of 8 types with a read whose node key is stale: a
/// graph-stage refusal, `invalid_package`/`stale-node-key`.
fn stale_chain_package() -> (Value, Value, CheckedPackageEvidence) {
    let read = || Read {
        owner: "C8",
        field: "f8",
        ty: Ty::Range(0, 1000),
    };
    let declared = [("C8", OBJECT)];
    let (valid, _, _) = package(&chain(8), &declared, &[read()]);
    let (mut stale, evidence, positions) = package(&chain(8), &declared, &[read()]);
    let at = positions.first().copied().expect("the read");
    stale["semantic_graph"]["nodes"][at]["node_id"]["digest"] = json!("f".repeat(64));
    rebuild_source_map(&mut stale);
    refresh_identity(&mut stale);
    (valid, stale, evidence)
}

/// Tables are built after every row's step 1 and before the graph stage: a
/// table charge that exhausts the limit together with a graph-stage refusal
/// (a stale node key) is `incomplete`, and a step 1 refusal at row 1 together
/// with a table charge that exhausts the limit at row 0 is the step 1 refusal.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_the_tables_follow_every_step_one_and_precede_the_graph_stage() {
    let (valid, stale, evidence) = stale_chain_package();
    // The smallest limit under which the tables are built: one below it, the
    // build is what exhausts the limit.
    let exhausts_tables = |package: &Value, work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        matches!(
            read_with(package, &evidence, limits),
            CheckedPackageV2ReadResult::Incomplete(incomplete)
                if incomplete.path.as_ref().map(ToString::to_string).as_deref()
                    == Some("/lock/model_selections/0")
        )
    };
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(!exhausts_tables(&stale, high));
    while low < high {
        let middle = low + (high - low) / 2;
        if exhausts_tables(&stale, middle) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    let smallest = low;
    assert!(smallest > 43, "the tables of 8 types are charged 43");
    // With room, the stale key is the refusal the graph stage draws.
    match read_with(&stale, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::StaleNodeKey)
            );
        }
        other => panic!("expected the stale-key refusal, got {other:?}"),
    }
    // One under, the same package is `incomplete` at the tables' row, not the
    // graph stage's stale key.
    assert!(exhausts_tables(&stale, smallest - 1));

    // Row 0 is the chain, row 1 a selection no evidence supplies.
    let mut two = valid;
    two["lock"]["model_selections"]
        .as_array_mut()
        .expect("selections")
        .push(json!({
            "identity": "acme/other", "digest_domain": "sha256-jcs", "digest": "b".repeat(64),
        }));
    refresh_identity(&mut two);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = smallest - 1;
    match read_with(&two, &evidence, limits) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
            assert_eq!(
                refusal.path.map(|path| path.to_string()).as_deref(),
                Some("/lock/model_selections/1/digest")
            );
        }
        other => panic!("expected the row 1 step 1 refusal, got {other:?}"),
    }
}

/// The order is the field name's, bytewise, where it differs from the order of
/// the member identities `<owner>/<name>`: an inherited `zeta` of `Base`
/// identifies before an own `alpha` of `Order`, and still comes after it. The
/// name lookup finds both, so it searches the order the list is in.
///
/// Trace: FR-038-AC-136
#[trace("TC-227", "FR-038-AC-136")]
#[test]
fn tc_227_the_order_is_by_name_where_identity_order_differs() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(
            "Base",
            &[],
            &[field("zeta", &thousand), field("mu", &thousand)],
        ),
        object(ORDER, &["Base"], &[field("alpha", &thousand)]),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    let fields = fields_of(&package, ORDER);
    assert_eq!(names(&fields), ["alpha", "mu", "zeta"]);
    for name in ["alpha", "mu", "zeta"] {
        assert_eq!(fields.field(name).map(|field| field.name()), Some(name));
    }
}

/// With two ambiguous names, the error names the smallest, so the answer does
/// not depend on the order of the declarations.
///
/// Trace: FR-038-AC-139
#[trace("TC-227", "FR-038-AC-139")]
#[test]
fn tc_227_the_ambiguous_name_the_error_carries_is_the_smallest() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object("Left", &[], &[field("b", &thousand), field("a", &thousand)]),
        object(
            ORDER,
            &["Left"],
            &[field("b", &thousand), field("a", &thousand)],
        ),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    assert_eq!(
        package.model_object_fields(&declaration_id(ORDER)),
        Err(CheckedModelFieldsError::AmbiguousField("a".into()))
    );
}

/// The members of the returned structs are reached through their accessors
/// alone: `name()` and `member_type()` of a field, `fields()` and `field()` of
/// the list. That no member is public is decided by the `compile_fail`
/// doctests of `CheckedModelField` and `CheckedModelObjectFields`, one probe
/// per member, which `make test` runs in its `--doc` lane.
///
/// Trace: FR-038-AC-142
#[trace("TC-227", "FR-038-AC-142")]
#[test]
fn tc_227_the_struct_members_are_reached_through_accessors() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(ORDER, &[], &[field("balance", &thousand)]),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    let fields = fields_of(&package, ORDER);
    let [balance] = fields.fields() else {
        panic!("one field");
    };
    assert_eq!(balance.name(), "balance");
    assert_eq!(balance.member_type(), Some(&range(0, 1000)));
    assert_eq!(fields.field("balance"), Some(balance));
}
