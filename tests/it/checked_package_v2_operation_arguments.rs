// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-448 (FR-347): a record value type that conforms to the record type an
//! operand position declares is admitted at an operation-call argument, a
//! field-read operand, a `case` arm body, a `union_value` payload, a function
//! or expression body root and a `function.call` argument, and every other
//! type stays exact.
//!
//! Every package is built here, node by node, with every key computed through
//! `quire-canonical` from the preimages of FR-322 and QSL FR-092 and over a
//! real Semantic IR 2.0.0 domain package document the lock selects. Nothing of
//! the reader's own derivation keys a node.

use crate::support::checked_package::{
    application_node_key, canonical, evidence_for, fixture_source, node_id, nominal_package,
    pointer, rebuild_source_map, refresh_identity, sha256_hex, source_owner,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const LEDGER: &str = "acme/ledger";
const VENDOR: &str = "acme/vendor";
const STRUCTURAL_NODE: &str = "quire.structural-node/v1";
const NATIVE: &str = "ix://quire/native/";

fn iri(identity: &str, short: &str) -> String {
    format!("ix://{identity}/{short}")
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term": "aggregate", "members": members})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(ty), "value_kind": value_kind, "value": value})
}

/// A node's key under QSL's `quire.structural-node/v1` preimage (FR-092).
fn structural(
    tag: &str,
    form: &str,
    semantic_type: Option<&str>,
    declaration: Option<&[&str]>,
    owner: Option<&Value>,
    body: &Value,
) -> String {
    let mut preimage = json!({
        "version": STRUCTURAL_NODE,
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": semantic_type.map(node_id),
        "declaration": declaration.map(|name| json!({"qualified_name": name})),
        "recursion": null,
        "body": body,
    });
    if let Some(owner) = owner {
        preimage["owner"] = owner.clone();
    }
    sha256_hex(&canonical(&preimage))
}

fn wire_node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    role: &str,
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    })
}

/// The targets FR-322's application-node dependency join names in `term`.
fn targets(term: &Value, out: &mut BTreeSet<String>) {
    match term["term"].as_str() {
        Some("reference") => {
            out.insert(
                term["target"]["digest"]
                    .as_str()
                    .expect("digest")
                    .to_owned(),
            );
        }
        Some("application") => {
            if let Some(declaration) = term["operation"]["member"]["declaration"]["digest"].as_str()
            {
                out.insert(declaration.to_owned());
            }
            for argument in term["arguments"].as_array().into_iter().flatten() {
                targets(argument, out);
            }
        }
        Some("aggregate") => {
            for member in term["members"].as_array().into_iter().flatten() {
                targets(member, out);
            }
        }
        Some("binding") => targets(&term["value"], out),
        _ => {}
    }
}

fn dependencies_of(body: &Value) -> Vec<String> {
    let mut out = BTreeSet::new();
    targets(body, &mut out);
    out.into_iter().collect()
}

// --- the domain package documents ---------------------------------------------------

const ONE: (u64, Option<u64>, bool, bool) = (1, Some(1), false, true);

fn multiplicity((lower, upper, ordered, unique): (u64, Option<u64>, bool, bool)) -> Value {
    let mut multiplicity = json!({"lower": lower, "ordered": ordered, "unique": unique});
    if let Some(upper) = upper {
        multiplicity["upper"] = json!(upper);
    }
    multiplicity
}

fn slot(type_ref: &str, shape: (u64, Option<u64>, bool, bool)) -> Value {
    json!({"typeRef": type_ref, "multiplicity": multiplicity(shape)})
}

fn declared_type(
    identity: &str,
    short: &str,
    kind: &str,
    supertypes: &[&str],
    fields: &[(&str, String)],
    operations: Vec<Value>,
) -> Value {
    let node = iri(identity, short);
    let fields: Vec<Value> = fields
        .iter()
        .map(|(name, type_ref)| {
            json!({
                "identity": format!("{node}/{name}"), "name": name, "typeRef": type_ref,
                "presence": "required", "nullable": false, "defaultKind": "none",
                "multiplicity": multiplicity(ONE),
            })
        })
        .collect();
    json!({
        "identity": node, "displayName": short,
        "kind": {"module": identity, "name": kind},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes.iter().map(|short| iri(identity, short)).collect::<Vec<_>>(),
        "fields": fields, "operations": operations,
    })
}

fn document(identity: &str, types: Vec<Value>) -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": identity, "version": "1.0.0"},
        "constructs": [
            {"kind": {"module": identity, "name": "entity"},
             "construct": {"meaning": "quire.meaning.model.object-type/v1"}},
            {"kind": {"module": identity, "name": "record"},
             "construct": {"meaning": "quire.meaning.model.record-value-type/v1"}},
            {"kind": {"module": identity, "name": "digit"},
             "construct": {"meaning": "quire.meaning.model.value-type/v1"}},
        ],
        "types": types,
    })
}

fn native(name: &str) -> String {
    format!("{NATIVE}{name}")
}

/// `Money`; `TaxedMoney` and `GiftTaxedMoney` below it; an unrelated
/// `Voucher`; the diamond `D` over `B` and `C` over `A`; `OrderPlaced` with
/// its child `ExpressOrderPlaced` and grandchild `RushOrderPlaced`; the
/// `Ledger` object (and its subtype) declaring every operation; and the
/// `Digit` value type `Int[0, 9]`.
fn ledger_document() -> Value {
    let amount = [("amount", native("Integer"))];
    let record = |short: &str, supertypes: &[&str], fields: &[(&str, String)]| {
        declared_type(LEDGER, short, "record", supertypes, fields, vec![])
    };
    let money = iri(LEDGER, "Money");
    let operation = |name: &str, params: Vec<Value>, returns: &str| {
        json!({
            "identity": iri(LEDGER, &format!("Ledger/{name}")),
            "params": params,
            "returns": slot(returns, ONE),
        })
    };
    let one = |type_ref: &str| slot(type_ref, ONE);
    let integer = native("Integer");
    let operations = vec![
        operation("charge", vec![one(&money)], &integer),
        operation("settle", vec![one(&iri(LEDGER, "TaxedMoney"))], &integer),
        operation(
            "grandsettle",
            vec![one(&iri(LEDGER, "GiftTaxedMoney"))],
            &integer,
        ),
        operation(
            "pair",
            vec![one(&money), one(&iri(LEDGER, "TaxedMoney"))],
            &integer,
        ),
        operation("take_int", vec![one(&integer)], &integer),
        operation("take_range", vec![one(&iri(LEDGER, "Digit"))], &integer),
        operation("take_bool", vec![one(&native("Boolean"))], &integer),
        operation("take_ref", vec![one(&iri(LEDGER, "Ledger"))], &integer),
        operation(
            "take_set",
            vec![slot(&money, (0, Some(1), false, true))],
            &integer,
        ),
        operation("over_a", vec![one(&iri(LEDGER, "A"))], &integer),
        operation("mint_money", vec![], &money),
        operation("mint_taxed", vec![], &iri(LEDGER, "TaxedMoney")),
        operation("mint_gift", vec![], &iri(LEDGER, "GiftTaxedMoney")),
        operation("mint_voucher", vec![], &iri(LEDGER, "Voucher")),
    ];
    let digit = json!({
        "identity": iri(LEDGER, "Digit"), "displayName": "Digit",
        "kind": {"module": LEDGER, "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": "integer",
        "constraints": [
            {"keyword": "min", "operands": {"value": "0"}},
            {"keyword": "max", "operands": {"value": "9"}},
        ],
    });
    document(
        LEDGER,
        vec![
            record("Money", &[], &amount),
            record("TaxedMoney", &["Money"], &[]),
            record("GiftTaxedMoney", &["TaxedMoney"], &[]),
            record("Voucher", &[], &[]),
            record("A", &[], &[]),
            record("B", &["A"], &[]),
            record("C", &["A"], &[]),
            record("D", &["B", "C"], &[]),
            record(
                "OrderPlaced",
                &[],
                &[
                    ("order_id", native("Integer")),
                    ("occurred_at", native("Integer")),
                ],
            ),
            record(
                "ExpressOrderPlaced",
                &["OrderPlaced"],
                &[("promised_by", native("Integer"))],
            ),
            record("RushOrderPlaced", &["ExpressOrderPlaced"], &[]),
            declared_type(LEDGER, "Ledger", "entity", &[], &[], operations),
            declared_type(LEDGER, "SubLedger", "entity", &["Ledger"], &[], vec![]),
            digit,
        ],
    )
}

/// A second selected document declaring its own `Money` and `TaxedMoney`.
fn vendor_document() -> Value {
    document(
        VENDOR,
        vec![
            declared_type(VENDOR, "Money", "record", &[], &[], vec![]),
            declared_type(VENDOR, "TaxedMoney", "record", &["Money"], &[], vec![]),
        ],
    )
}

// --- the package ------------------------------------------------------------------

/// A type, as a test states it; the node key is derived from it.
#[derive(Clone, Debug)]
enum Ty {
    Boolean,
    Integer,
    Range(i128, i128),
    /// A record value type of the ledger document.
    Rec(&'static str),
    /// A record value type of the vendor document.
    Foreign(&'static str),
    /// An object type of the ledger document, as the type itself.
    Object(&'static str),
    /// `Reference<O>` for an object type of the ledger document.
    Ref(&'static str),
    Option(Box<Ty>),
    /// A `Set<E>[lower, upper]`.
    Set(Box<Ty>, u64, u64),
    /// A source-declared `composite_type`/`record` node named `name`.
    SourceRecord(&'static str),
    /// A raw node key.
    Key(String),
}

struct Scene {
    documents: Vec<Value>,
    nodes: Vec<Value>,
    seen: BTreeSet<String>,
}

fn digest_of(node: &Value) -> String {
    node["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned()
}

fn bounds_body(integer: &str, lower: &str, upper: &str) -> Value {
    let bound = |name: &str, value: &str| binding(name, literal(integer, "integer", value));
    aggregate(vec![bound("min", lower), bound("max", upper)])
}

impl Scene {
    fn new(documents: Vec<Value>) -> Self {
        Self {
            documents,
            nodes: Vec::new(),
            seen: BTreeSet::new(),
        }
    }

    fn add(&mut self, node: Value) -> String {
        let key = digest_of(&node);
        if self.seen.insert(key.clone()) {
            self.nodes.push(node);
        }
        key
    }

    fn scalar(&mut self, form: &str) -> String {
        let key = structural("scalar_type", form, None, None, None, &empty());
        self.add(wire_node(
            &key,
            "scalar_type",
            form,
            &key,
            &[],
            "type",
            empty(),
        ))
    }

    /// The model declaration node of `short` in the document `identity`.
    fn model(&mut self, identity: &str, short: &str, form: &str) -> String {
        let owner = json!({"kind": "model", "identity": identity, "node": iri(identity, short)});
        let key = structural("model", form, None, None, Some(&owner), &empty());
        let mut node = wire_node(&key, "model", form, &key, &[], "type", empty());
        node["owner"] = owner;
        self.add(node)
    }

    fn ty(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Boolean => self.scalar("boolean"),
            Ty::Integer => self.scalar("integer"),
            Ty::Range(lower, upper) => {
                let integer = self.scalar("integer");
                let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
                let key = structural(
                    "bounded_domain",
                    "integer_range",
                    Some(&integer),
                    None,
                    None,
                    &body,
                );
                self.add(wire_node(
                    &key,
                    "bounded_domain",
                    "integer_range",
                    &integer,
                    &[&integer],
                    "type",
                    body,
                ))
            }
            Ty::Rec(short) => self.model(LEDGER, short, "record_value_type"),
            Ty::Foreign(short) => self.model(VENDOR, short, "record_value_type"),
            Ty::Object(short) => self.model(LEDGER, short, "object_type"),
            Ty::Ref(short) => {
                let target = self.model(LEDGER, short, "object_type");
                self.composite("reference", &target)
            }
            Ty::Option(element) => {
                let element = self.ty(element);
                self.composite("option", &element)
            }
            Ty::Set(element, lower, upper) => {
                let element = self.ty(element);
                let collection = self.composite("set", &element);
                let integer = self.scalar("integer");
                let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
                let key = structural(
                    "bounded_domain",
                    "collection_bounds",
                    Some(&collection),
                    None,
                    None,
                    &body,
                );
                self.add(wire_node(
                    &key,
                    "bounded_domain",
                    "collection_bounds",
                    &collection,
                    &[&collection],
                    "type",
                    body,
                ))
            }
            Ty::SourceRecord(name) => {
                let integer = self.scalar("integer");
                let body = aggregate(vec![binding("amount", reference(&integer))]);
                let owner = source_owner(&fixture_source());
                let key = structural(
                    "composite_type",
                    "record",
                    None,
                    Some(&[name]),
                    Some(&owner),
                    &body,
                );
                let mut node = wire_node(
                    &key,
                    "composite_type",
                    "record",
                    &key,
                    &[&integer],
                    "declaration",
                    body,
                );
                node["declaration"] = json!({"qualified_name": [name]});
                node["owner"] = owner;
                self.add(node)
            }
            Ty::Key(key) => key.clone(),
        }
    }

    fn composite(&mut self, form: &str, element: &str) -> String {
        let body = aggregate(vec![reference(element)]);
        let key = structural("composite_type", form, None, None, None, &body);
        self.add(wire_node(
            &key,
            "composite_type",
            form,
            &key,
            &[element],
            "type",
            body,
        ))
    }

    /// A `value`/`parameter` node named `name` typed `ty`.
    fn param(&mut self, name: &str, ty: &Ty) -> String {
        let ty = self.ty(ty);
        let text = self.scalar("text");
        let integer = self.scalar("integer");
        let body = aggregate(vec![
            binding("name", literal(&text, "text", name)),
            binding("level", literal(&integer, "integer", "0")),
        ]);
        let key = structural("value", "parameter", Some(&ty), None, None, &body);
        self.add(wire_node(
            &key,
            "value",
            "parameter",
            &ty,
            &[],
            "expression",
            body,
        ))
    }

    /// An application node over `arguments`, typed `semantic` with the
    /// application's `result_type` `result`.
    fn application(&mut self, app: App<'_>) -> String {
        let body = json!({
            "term": "application",
            "operator": app.operator,
            "operation": {
                "identity": app.identity, "laws": [], "mode": null,
                "member": app.member, "leaves": [],
            },
            "result_type": node_id(app.result),
            "arguments": app.arguments,
        });
        let key = application_node_key(app.tag, app.form, app.semantic, &body);
        let dependencies = dependencies_of(&body);
        let dependencies: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        self.add(wire_node(
            &key,
            app.tag,
            app.form,
            app.semantic,
            &dependencies,
            "expression",
            body,
        ))
    }

    /// `receiver.name(arguments)`: a `dispatch_call` of the operation `name`
    /// declared by `declaring`, typed at `result`.
    fn dispatch(&mut self, declaring: &str, name: &str, params: &[String], result: &str) -> String {
        self.application(App {
            tag: "expression",
            form: "call",
            operator: "call",
            identity: "quire.op.model.dispatch_call",
            member: json!({"kind": "operation", "declaration": node_id(declaring), "name": name}),
            semantic: result,
            result,
            arguments: params.iter().map(|key| reference(key)).collect(),
        })
    }

    /// A source-declared `function`/`pure_function` of `semantic` whose body
    /// is the aggregate of `members`.
    fn function(&mut self, name: &str, semantic: &str, members: Vec<Value>) -> String {
        let body = aggregate(members);
        let owner = source_owner(&fixture_source());
        let key = structural(
            "function",
            "pure_function",
            Some(semantic),
            Some(&[name]),
            Some(&owner),
            &body,
        );
        let dependencies: Vec<String> = dependencies_of(&body)
            .into_iter()
            .filter(|key| self.seen.contains(key))
            .collect();
        let dependencies: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        let mut node = wire_node(
            &key,
            "function",
            "pure_function",
            semantic,
            &dependencies,
            "declaration",
            body,
        );
        node["declaration"] = json!({"qualified_name": [name]});
        node["owner"] = owner;
        self.add(node)
    }

    /// A source-declared union type of `members` (name, payload type keys).
    fn union(&mut self, name: &str, members: &[(&str, Vec<String>)]) -> String {
        let body = aggregate(
            members
                .iter()
                .map(|(member, payload)| {
                    binding(
                        member,
                        aggregate(payload.iter().map(|ty| reference(ty)).collect()),
                    )
                })
                .collect(),
        );
        let owner = source_owner(&fixture_source());
        let key = structural(
            "composite_type",
            "union",
            None,
            Some(&[name]),
            Some(&owner),
            &body,
        );
        let mut payload: Vec<&str> = members
            .iter()
            .flat_map(|(_, payload)| payload.iter().map(String::as_str))
            .collect();
        payload.sort_unstable();
        payload.dedup();
        let mut node = wire_node(
            &key,
            "composite_type",
            "union",
            &key,
            &payload,
            "declaration",
            body,
        );
        node["declaration"] = json!({"qualified_name": [name]});
        node["owner"] = owner;
        self.add(node)
    }

    /// `ty::member(payload...)`, the payload terms value nodes.
    fn union_value(&mut self, ty: &str, member: &str, payload: &[String]) -> String {
        let body = aggregate(vec![binding(
            member,
            aggregate(payload.iter().map(|key| reference(key)).collect()),
        )]);
        let key = application_node_key("value", "union_value", ty, &body);
        let dependencies = dependencies_of(&body);
        let dependencies: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        self.add(wire_node(
            &key,
            "value",
            "union_value",
            ty,
            &dependencies,
            "expression",
            body,
        ))
    }

    fn finish(self) -> Case {
        let mut documents = self.documents;
        documents.sort_by_key(|document| {
            document["package"]["identity"]
                .as_str()
                .expect("identity")
                .to_owned()
        });
        let mut package = nominal_package(&[]);
        package["lock"]["model_selections"] = Value::Array(
            documents
                .iter()
                .map(|document| {
                    json!({
                        "identity": document["package"]["identity"],
                        "digest_domain": "sha256-jcs",
                        "digest": sha256_hex(&canonical(document)),
                    })
                })
                .collect(),
        );
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .extend(self.nodes);
        rebuild_source_map(&mut package);
        refresh_identity(&mut package);
        let mut evidence = evidence_for(&package);
        for document in &documents {
            evidence.insert_domain_package_document(
                sha256_hex(&canonical(document)),
                canonical(document),
            );
        }
        Case {
            package,
            evidence,
            node: 0,
        }
    }
}

struct App<'a> {
    tag: &'a str,
    form: &'a str,
    operator: &'a str,
    identity: &'a str,
    member: Value,
    semantic: &'a str,
    result: &'a str,
    arguments: Vec<Value>,
}

/// A package, its evidence and the position of the node under test.
struct Case {
    package: Value,
    evidence: CheckedPackageEvidence,
    node: usize,
}

impl Case {
    /// The case whose node under test is `key`.
    fn at(mut self, key: &str) -> Self {
        self.node = self.package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"]["digest"] == key)
            .expect("the node under test is in the graph");
        self
    }

    fn read_at(&self, work: Option<u64>) -> CheckedPackageV2ReadResult {
        let mut limits = CheckedPackageReadLimits::bounded();
        if let Some(work) = work {
            limits.work = work;
        }
        CheckedPackageV2::read(&canonical(&self.package), limits, &self.evidence)
    }

    fn read(&self) -> CheckedPackageV2ReadResult {
        self.read_at(None)
    }

    fn admits(&self, label: &str) {
        match self.read() {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{label}: expected admission, read {other:?}"),
        }
    }

    /// The read refuses with `code`/`cause` at `tail` below the node under test.
    fn refuses(&self, label: &str, code: Code, cause: Cause, tail: &str) {
        let CheckedPackageV2ReadResult::Refused(refusal) = self.read() else {
            panic!("{label}: expected a refusal");
        };
        let expected = format!("/semantic_graph/nodes/{}{tail}", self.node);
        assert_eq!(
            (refusal.code, refusal.cause, refusal.path.clone()),
            (code, Some(cause), Some(pointer(&expected))),
            "{label}: {refusal:?}"
        );
    }

    fn ineligible(&self, label: &str, tail: &str) {
        self.refuses(label, Code::IllTyped, Cause::OperatorIneligible, tail);
    }

    fn mismatch(&self, label: &str) {
        self.refuses(label, Code::IllTyped, Cause::TypeMismatch, "");
    }

    /// The least `work` limit under which the read admits.
    fn least_work(&self) -> u64 {
        let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
        assert!(
            matches!(
                self.read_at(Some(high)),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "the case admits under the default work limit"
        );
        while low < high {
            let middle = low + (high - low) / 2;
            match self.read_at(Some(middle)) {
                CheckedPackageV2ReadResult::Admitted(_) => high = middle,
                CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
                other => panic!("expected admission or the work limit, read {other:?}"),
            }
        }
        low
    }
}

fn ledger_scene() -> Scene {
    Scene::new(vec![ledger_document()])
}

/// The result type each operation of the ledger declares.
fn result_of(operation: &str) -> Ty {
    match operation {
        "mint_money" => Ty::Rec("Money"),
        "mint_taxed" => Ty::Rec("TaxedMoney"),
        "mint_gift" => Ty::Rec("GiftTaxedMoney"),
        "mint_voucher" => Ty::Rec("Voucher"),
        _ => Ty::Integer,
    }
}

/// `receiver.operation(arguments)`, the arguments parameters typed as given.
/// Every case holds the same parameter nodes, so every case costs the same
/// work apart from the conformance walk.
fn call_with(
    scene: &mut Scene,
    receiver: &Ty,
    operation: &str,
    arguments: &[Ty],
    result: &Ty,
) -> String {
    for (name, ty) in [
        ("pad_money", Ty::Rec("Money")),
        ("pad_taxed", Ty::Rec("TaxedMoney")),
        ("pad_gift", Ty::Rec("GiftTaxedMoney")),
        ("pad_d", Ty::Rec("D")),
        ("pad_a", Ty::Rec("A")),
    ] {
        scene.param(name, &ty);
    }
    let declaring = scene.ty(&Ty::Object("Ledger"));
    let mut params = vec![scene.param("receiver", receiver)];
    for (position, ty) in arguments.iter().enumerate() {
        params.push(scene.param(&format!("arg{position}"), ty));
    }
    let result = scene.ty(result);
    scene.dispatch(&declaring, operation, &params, &result)
}

fn call(operation: &str, arguments: &[Ty]) -> Case {
    call_over(&Ty::Ref("Ledger"), operation, arguments)
}

fn call_over(receiver: &Ty, operation: &str, arguments: &[Ty]) -> Case {
    let mut scene = ledger_scene();
    let key = call_with(
        &mut scene,
        receiver,
        operation,
        arguments,
        &result_of(operation),
    );
    scene.finish().at(&key)
}

const ARG_1: &str = "/body/arguments/1";
const ARG_2: &str = "/body/arguments/2";

// --- TC-448: operation-call arguments (FR-347-AC-1 to AC-7) ---------------------------

/// Trace: FR-347-AC-1
#[trace("TC-448", "FR-347-AC-1")]
#[test]
fn tc_448_a_child_and_a_grandchild_record_are_admitted_at_a_parent_parameter() {
    for argument in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        let case = call("charge", &[Ty::Rec(argument)]);
        case.admits(argument);
        // The conformance is the document's: the package carries neither a
        // `supertypes` member nor a conversion node.
        let text = String::from_utf8(canonical(&case.package)).expect("utf-8");
        assert!(!text.contains("supertypes"), "{argument}");
        assert!(!text.contains("conversion"), "{argument}");
    }
}

/// Trace: FR-347-AC-2
#[trace("TC-448", "FR-347-AC-2")]
#[test]
fn tc_448_an_unrelated_record_is_refused_at_its_argument() {
    call("charge", &[Ty::Rec("Voucher")]).ineligible("Voucher at Money", ARG_1);
}

/// Trace: FR-347-AC-3
#[trace("TC-448", "FR-347-AC-3")]
#[test]
fn tc_448_an_ancestor_never_conforms_to_a_descendant() {
    call("settle", &[Ty::Rec("Money")]).ineligible("Money at TaxedMoney", ARG_1);
    call("grandsettle", &[Ty::Rec("TaxedMoney")]).ineligible("TaxedMoney at Gift", ARG_1);
    call("grandsettle", &[Ty::Rec("Money")]).ineligible("Money at Gift", ARG_1);
    call("settle", &[Ty::Rec("GiftTaxedMoney")]).admits("Gift at TaxedMoney");
    call("settle", &[Ty::Rec("TaxedMoney")]).admits("TaxedMoney at TaxedMoney");
    call("grandsettle", &[Ty::Rec("GiftTaxedMoney")]).admits("Gift at Gift");
}

/// Trace: FR-347-AC-4
#[trace("TC-448", "FR-347-AC-4")]
#[test]
fn tc_448_every_other_parameter_type_is_compared_exactly() {
    let money_set = |element: &'static str| Ty::Set(Box::new(Ty::Rec(element)), 0, 1);
    let admitted = [
        ("take_int", Ty::Integer),
        ("take_range", Ty::Range(0, 9)),
        ("take_bool", Ty::Boolean),
        ("take_ref", Ty::Ref("Ledger")),
        ("take_set", money_set("Money")),
    ];
    for (operation, argument) in admitted {
        call(operation, &[argument]).admits(operation);
    }
    let refused = [
        ("take_int", Ty::Rec("TaxedMoney")),
        ("take_range", Ty::Range(0, 5)),
        ("take_bool", Ty::Integer),
        ("take_ref", Ty::Ref("SubLedger")),
        ("take_set", Ty::Rec("TaxedMoney")),
        ("take_set", money_set("TaxedMoney")),
        ("charge", Ty::Option(Box::new(Ty::Rec("Money")))),
    ];
    for (operation, argument) in refused {
        let label = format!("{operation} over {argument:?}");
        call(operation, std::slice::from_ref(&argument)).ineligible(&label, ARG_1);
    }
}

/// Trace: FR-347-AC-5
#[trace("TC-448", "FR-347-AC-5")]
#[test]
fn tc_448_ancestry_is_read_from_the_selected_document_only() {
    // A source-declared record with `TaxedMoney`'s name and fields.
    call("charge", &[Ty::SourceRecord("TaxedMoney")]).ineligible("source record at Money", ARG_1);
    // `TaxedMoney` of a second selected document does not conform to this
    // document's `Money`.
    let mut scene = Scene::new(vec![ledger_document(), vendor_document()]);
    let key = call_with(
        &mut scene,
        &Ty::Ref("Ledger"),
        "charge",
        &[Ty::Foreign("TaxedMoney")],
        &Ty::Integer,
    );
    scene
        .finish()
        .at(&key)
        .ineligible("foreign TaxedMoney", ARG_1);
}

/// Trace: FR-347-AC-6
#[trace("TC-448", "FR-347-AC-6")]
#[test]
fn tc_448_the_first_offending_argument_in_position_order_is_reported() {
    let (money, taxed, voucher) = (Ty::Rec("Money"), Ty::Rec("TaxedMoney"), Ty::Rec("Voucher"));
    call("pair", &[taxed.clone(), taxed.clone()]).admits("both conforming");
    call("pair", &[taxed.clone(), money.clone()]).ineligible("conforming then ancestor", ARG_2);
    call("pair", &[money.clone(), money.clone()]).ineligible("exact then ancestor", ARG_2);
    call("pair", &[voucher.clone(), taxed.clone()]).ineligible("unrelated then conforming", ARG_1);
    call("pair", &[voucher, money]).ineligible("both non-conforming", ARG_1);
    // The arity check precedes every argument check.
    let arguments = "/body/arguments";
    call("charge", &[Ty::Rec("Voucher"), Ty::Rec("Money")]).ineligible("one too many", arguments);
    call("charge", &[]).ineligible("one too few", arguments);
}

/// Trace: FR-347-AC-7
#[trace("TC-448", "FR-347-AC-7")]
#[test]
fn tc_448_the_receiver_and_the_result_type_stay_exact() {
    call_over(&Ty::Ref("SubLedger"), "charge", &[Ty::Rec("Money")])
        .ineligible("Reference of a subtype", "/body/arguments/0");
    let mut scene = ledger_scene();
    let key = call_with(
        &mut scene,
        &Ty::Ref("Ledger"),
        "charge",
        &[Ty::Rec("Money")],
        &Ty::Boolean,
    );
    scene
        .finish()
        .at(&key)
        .ineligible("result_type differs", "/body/result_type");
}

// --- TC-448: work accounting (FR-347-AC-8) -------------------------------------------

/// Trace: FR-347-AC-8
#[trace("TC-448", "FR-347-AC-8")]
#[test]
fn tc_448_conformance_charges_every_reachable_supertype_edge_once() {
    let charged =
        |operation: &str, exact: &'static str, conforming: &'static str, edges: u64, row: bool| {
            let control = call(operation, &[Ty::Rec(exact)]);
            let walked = call(operation, &[Ty::Rec(conforming)]);
            let (baseline, total) = (control.least_work(), walked.least_work());
            assert_eq!(
                total - baseline,
                edges,
                "{conforming} at {exact}: the walk charges {edges}"
            );
            // One unit below the measured total is the work limit; the total admits.
            match walked.read_at(Some(total - 1)) {
                CheckedPackageV2ReadResult::Incomplete(incomplete) => {
                    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
                }
                other => panic!("{conforming} at {exact}: expected the work limit, read {other:?}"),
            }
            assert!(matches!(
                walked.read_at(Some(total)),
                CheckedPackageV2ReadResult::Admitted(_)
            ));
            // The walk is charged at the selection row of the document that
            // declares the expected record: that many more limits run out there.
            let at_row = |case: &Case, total: u64| {
                (0..total)
                    .filter(|limit| match case.read_at(Some(*limit)) {
                        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
                            incomplete.path.map(|path| path.to_string()).as_deref()
                                == Some("/lock/model_selections/0")
                        }
                        _ => false,
                    })
                    .count()
            };
            if !row {
                return;
            }
            let edges_at_row = at_row(&walked, total) - at_row(&control, baseline);
            assert_eq!(
                edges_at_row as u64, edges,
                "{conforming} at {exact}: the walk is charged at the selection row"
            );
        };
    // The walk does not stop at the parameter's record.
    charged("charge", "Money", "GiftTaxedMoney", 2, true);
    charged("settle", "TaxedMoney", "GiftTaxedMoney", 2, false);
    // A diamond costs its four distinct edges.
    charged("over_a", "A", "D", 4, false);
}

// --- TC-448: field reads (FR-347-AC-9) ------------------------------------------------

/// `operand.field` as a `quire.op.record.project` of `field` declared by
/// `declaring`, the operand a parameter typed `operand`.
fn read_of(declaring: &Ty, field: &str, operand: &Ty) -> Case {
    let mut scene = ledger_scene();
    let declaring = scene.ty(declaring);
    // An object operand is the declaration node itself: no parameter is typed
    // at an object type.
    let operand = match operand {
        Ty::Object(_) => scene.ty(operand),
        _ => scene.param("operand", operand),
    };
    let integer = scene.scalar("integer");
    let key = scene.application(App {
        tag: "expression",
        form: "query",
        operator: "query",
        identity: "quire.op.record.project",
        member: json!({"kind": "field", "declaration": node_id(&declaring), "name": field}),
        semantic: &integer,
        result: &integer,
        arguments: vec![reference(&operand)],
    });
    scene.finish().at(&key)
}

/// Trace: FR-347-AC-9
#[trace("TC-448", "FR-347-AC-9")]
#[test]
fn tc_448_a_record_field_is_read_over_a_conforming_operand() {
    let placed = Ty::Rec("OrderPlaced");
    let express = Ty::Rec("ExpressOrderPlaced");
    for operand in ["OrderPlaced", "ExpressOrderPlaced", "RushOrderPlaced"] {
        read_of(&placed, "occurred_at", &Ty::Rec(operand)).admits(operand);
    }
    // An inherited field, named on the child.
    read_of(&express, "order_id", &Ty::Rec("ExpressOrderPlaced")).admits("inherited");
    read_of(&express, "order_id", &Ty::Rec("RushOrderPlaced")).admits("inherited, grandchild");
    read_of(&placed, "occurred_at", &Ty::Rec("Voucher"))
        .ineligible("unrelated record", "/body/arguments/0");
    read_of(&placed, "occurred_at", &Ty::Object("Ledger"))
        .ineligible("object operand", "/body/arguments/0");
    // The child's operand does not give `OrderPlaced` the child's field.
    read_of(&placed, "promised_by", &Ty::Rec("ExpressOrderPlaced"))
        .ineligible("descendant-only field", "/body/operation/member/name");
    read_of(&express, "promised_by", &Ty::Rec("ExpressOrderPlaced")).admits("own field");
    // A record owns no operation.
    let mut scene = ledger_scene();
    let record = scene.ty(&placed);
    let operand = scene.param("operand", &placed);
    let integer = scene.scalar("integer");
    let key = scene.dispatch(&record, "occurred_at", &[operand], &integer);
    scene
        .finish()
        .at(&key)
        .ineligible("operation naming a record", "/body/operation/member/name");
}

// --- TC-448: case arms and union payloads (FR-347-AC-10, AC-11) -------------------

/// A `case` over `Payment { Pay(Money), Alt(Money) }` whose arm bodies are
/// parameters typed `first` and `second`, whose binders are typed `binder`
/// (`Money` when it conforms) and whose result is `result`.
fn case_over(first: &Ty, second: &Ty, binder: &Ty, result: &Ty) -> Case {
    let mut scene = ledger_scene();
    let money = scene.ty(&Ty::Rec("Money"));
    let payment = scene.union(
        "Payment",
        &[("Pay", vec![money.clone()]), ("Alt", vec![money])],
    );
    let scrutinee = scene.param("scrutinee", &Ty::Key(payment));
    let arm = |scene: &mut Scene, member: &str, name: &str, binder_ty: &Ty, body: &Ty| {
        let binder = scene.param(&format!("binder_{name}"), binder_ty);
        let body = scene.param(&format!("body_{name}"), body);
        binding(
            member,
            aggregate(vec![aggregate(vec![reference(&binder)]), reference(&body)]),
        )
    };
    let pay = arm(&mut scene, "Pay", "pay", binder, first);
    let alt = arm(&mut scene, "Alt", "alt", &Ty::Rec("Money"), second);
    let result = scene.ty(result);
    let key = scene.application(App {
        tag: "expression",
        form: "case",
        operator: "case",
        identity: "quire.op.control.case",
        member: Value::Null,
        semantic: &result,
        result: &result,
        arguments: vec![reference(&scrutinee), pay, alt],
    });
    scene.finish().at(&key)
}

/// Trace: FR-347-AC-10
#[trace("TC-448", "FR-347-AC-10")]
#[test]
fn tc_448_a_case_arm_body_conforms_to_the_result_record() {
    let (money, taxed) = (Ty::Rec("Money"), Ty::Rec("TaxedMoney"));
    case_over(&taxed, &money, &money, &money).admits("TaxedMoney and Money at Money");
    case_over(&money, &money, &money, &money).admits("exact");
    case_over(&Ty::Rec("Voucher"), &money, &money, &taxed)
        .refuses_at_node("Voucher arm at TaxedMoney");
    case_over(&money, &taxed, &money, &taxed).refuses_at_node("Money arm at TaxedMoney");
    // A binder is bound to the payload type: it stays exact.
    case_over(&money, &money, &taxed, &money).refuses_at_node("child-typed binder");
    // A non-record result type stays exact.
    case_over(&Ty::Integer, &Ty::Integer, &money, &Ty::Integer).admits("Integer arms");
    case_over(&taxed, &taxed, &money, &Ty::Integer).refuses_at_node("record arm at Integer");
}

impl Case {
    /// `ill_typed`/`operator-ineligible` at the node itself.
    fn refuses_at_node(&self, label: &str) {
        self.ineligible(label, "");
    }
}

/// `Pay(payload)` of a union declaring a `declared` payload.
fn payload_of(declared: &'static str, payload: &Ty) -> Case {
    let mut scene = ledger_scene();
    let declared = scene.ty(&Ty::Rec(declared));
    let union = scene.union("Payment", &[("Pay", vec![declared])]);
    let term = scene.param("term", payload);
    let key = scene.union_value(&union, "Pay", &[term]);
    scene.finish().at(&key)
}

/// Trace: FR-347-AC-11
#[trace("TC-448", "FR-347-AC-11")]
#[test]
fn tc_448_a_union_payload_conforms_to_the_declared_record() {
    for payload in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        payload_of("Money", &Ty::Rec(payload)).admits(payload);
    }
    payload_of("TaxedMoney", &Ty::Rec("Voucher")).mismatch("Voucher at TaxedMoney");
    payload_of("TaxedMoney", &Ty::Rec("Money")).mismatch("Money at TaxedMoney");
    // A non-record payload stays exact.
    payload_of("Money", &Ty::Integer).mismatch("Integer at Money");
}

// --- TC-448: body roots (FR-347-AC-12) ------------------------------------------------

/// A `function` of declared type `declared` whose `body` binding references
/// a root parameter typed `root`.
fn function_root(declared: &'static str, root: &Ty) -> Case {
    let mut scene = ledger_scene();
    let declared = scene.ty(&Ty::Rec(declared));
    let root = scene.param("root", root);
    let key = scene.function(
        "produce",
        &declared,
        vec![
            binding("parameters", aggregate(vec![reference(&root)])),
            binding("body", reference(&root)),
        ],
    );
    scene.finish().at(&key)
}

/// A node of declared type `declared` whose body is the application
/// `mint_<root>()`, tagged `expression` or `function`.
fn application_root(tag: &str, declared: &'static str, root: &str) -> Case {
    let mut scene = ledger_scene();
    let declared = scene.ty(&Ty::Rec(declared));
    let ledger = scene.ty(&Ty::Object("Ledger"));
    let receiver = scene.param("receiver", &Ty::Ref("Ledger"));
    let result = scene.ty(&result_of(root));
    let (form, role_tag) = if tag == "function" {
        ("pure_function", "function")
    } else {
        ("call", "expression")
    };
    let key = scene.application(App {
        tag: role_tag,
        form,
        operator: "call",
        identity: "quire.op.model.dispatch_call",
        member: json!({"kind": "operation", "declaration": node_id(&ledger), "name": root}),
        semantic: &declared,
        result: &result,
        arguments: vec![reference(&receiver)],
    });
    scene.finish().at(&key)
}

/// Trace: FR-347-AC-12
#[trace("TC-448", "FR-347-AC-12")]
#[test]
fn tc_448_a_body_root_conforms_to_the_declared_record() {
    for root in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        function_root("Money", &Ty::Rec(root)).admits(root);
    }
    function_root("Money", &Ty::Rec("Voucher")).mismatch("Voucher root");
    function_root("TaxedMoney", &Ty::Rec("Money")).mismatch("ancestor root");
    function_root("Money", &Ty::Integer).mismatch("non-record root at a record");
    for tag in ["expression", "function"] {
        for root in ["mint_money", "mint_taxed", "mint_gift"] {
            application_root(tag, "Money", root).admits(&format!("{tag} {root}"));
        }
        application_root(tag, "Money", "mint_voucher").mismatch(&format!("{tag} Voucher"));
        application_root(tag, "TaxedMoney", "mint_money").mismatch(&format!("{tag} ancestor"));
    }
    // A non-record declared type stays exact.
    let mut scene = ledger_scene();
    let integer = scene.ty(&Ty::Integer);
    let root = scene.param("root", &Ty::Rec("Money"));
    let key = scene.function(
        "produce",
        &integer,
        vec![
            binding("parameters", aggregate(vec![reference(&root)])),
            binding("body", reference(&root)),
        ],
    );
    scene.finish().at(&key).mismatch("record root at Integer");
}

// --- TC-448: function.call arguments (FR-347-AC-13, AC-14) ------------------------

/// A `function.call` of the callee `f` with `arguments` parameters typed as
/// given. `f` takes `signature` parameters and binds the `members` after the
/// `parameters` binding the `bindings` closure builds from their keys.
fn function_call(
    signature: &[Ty],
    arguments: &[Ty],
    bindings: impl FnOnce(&mut Scene, &[String]) -> Vec<Value>,
) -> Case {
    let mut scene = ledger_scene();
    let integer = scene.ty(&Ty::Integer);
    let declared: Vec<String> = signature
        .iter()
        .enumerate()
        .map(|(position, ty)| scene.param(&format!("p{position}"), ty))
        .collect();
    let members = bindings(&mut scene, &declared);
    let callee = scene.function("f", &integer, members);
    let mut terms = vec![reference(&callee)];
    for (position, ty) in arguments.iter().enumerate() {
        let argument = scene.param(&format!("a{position}"), ty);
        terms.push(reference(&argument));
    }
    let key = scene.application(App {
        tag: "expression",
        form: "call",
        operator: "call",
        identity: "quire.op.function.call",
        member: Value::Null,
        semantic: &integer,
        result: &integer,
        arguments: terms,
    });
    scene.finish().at(&key)
}

/// The well-formed members: `parameters` listing every declared parameter and
/// a `body` reading the last one.
fn well_formed(_: &mut Scene, declared: &[String]) -> Vec<Value> {
    vec![
        binding(
            "parameters",
            aggregate(declared.iter().map(|key| reference(key)).collect()),
        ),
        binding("body", reference(declared.last().expect("a parameter"))),
    ]
}

/// Trace: FR-347-AC-13
#[trace("TC-448", "FR-347-AC-13")]
#[test]
fn tc_448_function_call_arguments_follow_the_callee_parameters() {
    let signature = [Ty::Rec("Money"), Ty::Integer];
    for first in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        function_call(&signature, &[Ty::Rec(first), Ty::Integer], well_formed).admits(first);
    }
    let arguments = "/body/arguments";
    function_call(&signature, &[Ty::Rec("Money")], well_formed)
        .ineligible("one argument", arguments);
    function_call(
        &signature,
        &[Ty::Rec("Money"), Ty::Integer, Ty::Integer],
        well_formed,
    )
    .ineligible("three arguments", arguments);
    function_call(&signature, &[Ty::Rec("Voucher"), Ty::Integer], well_formed)
        .ineligible("Voucher", ARG_1);
    function_call(
        &signature,
        &[Ty::Rec("Money"), Ty::Range(0, 5)],
        well_formed,
    )
    .ineligible("Int[0, 5] at Integer", ARG_2);
    function_call(
        &[Ty::Rec("TaxedMoney"), Ty::Integer],
        &[Ty::Rec("Money"), Ty::Integer],
        well_formed,
    )
    .ineligible("ancestor", ARG_1);
    // A parameter the body never reads still counts.
    let unread = [Ty::Rec("Money"), Ty::Integer, Ty::Integer];
    let reads_one = |_: &mut Scene, declared: &[String]| {
        vec![
            binding(
                "parameters",
                aggregate(declared.iter().map(|key| reference(key)).collect()),
            ),
            binding("body", reference(&declared[1])),
        ]
    };
    function_call(&unread, &[Ty::Rec("Money"), Ty::Integer], reads_one)
        .ineligible("two arguments, three parameters", arguments);
    function_call(
        &unread,
        &[Ty::Rec("Money"), Ty::Integer, Ty::Integer],
        reads_one,
    )
    .admits("three arguments, three parameters");
    // A callee without the binding takes no argument check.
    let unbound = |_: &mut Scene, declared: &[String]| {
        vec![binding(
            "body",
            reference(declared.last().expect("a parameter")),
        )]
    };
    function_call(
        &signature,
        &[Ty::Rec("Voucher"), Ty::Integer, Ty::Integer],
        unbound,
    )
    .admits("no parameters binding");
}

fn parameters_binding(entries: Vec<Value>) -> Value {
    binding("parameters", aggregate(entries))
}

/// Trace: FR-347-AC-14
#[trace("TC-448", "FR-347-AC-14")]
#[test]
fn tc_448_a_malformed_parameters_binding_is_refused_at_the_callee() {
    let signature = [Ty::Rec("Money"), Ty::Integer];
    let arguments = [Ty::Rec("Money"), Ty::Integer];
    let callee = "/body/arguments/0";
    type Members = Box<dyn Fn(&mut Scene, &[String]) -> Vec<Value>>;
    let cases: Vec<(&str, Members)> = vec![
        (
            "second in the body",
            Box::new(|_, declared| {
                vec![
                    binding("body", reference(&declared[1])),
                    parameters_binding(declared.iter().map(|key| reference(key)).collect()),
                ]
            }),
        ),
        (
            "not an aggregate",
            Box::new(|_, declared| {
                vec![
                    binding("parameters", reference(&declared[0])),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
        (
            "a literal entry",
            Box::new(|scene, declared| {
                let integer = scene.scalar("integer");
                vec![
                    parameters_binding(vec![
                        literal(&integer, "integer", "1"),
                        reference(&declared[1]),
                    ]),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
        (
            "an absent target",
            Box::new(|_, declared| {
                let absent = sha256_hex(b"absent");
                vec![
                    parameters_binding(vec![reference(&absent), reference(&declared[1])]),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
        (
            "a non-parameter target",
            Box::new(|scene, declared| {
                let integer = scene.scalar("integer");
                vec![
                    parameters_binding(vec![reference(&integer), reference(&declared[1])]),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
    ];
    for (label, members) in cases {
        function_call(&signature, &arguments, members).ineligible(label, callee);
    }
}

// --- the widening stops at record value types --------------------------------

/// Trace: FR-347-AC-10, FR-347-AC-13
#[trace("TC-448", "FR-347-AC-10", "FR-347-AC-13")]
#[test]
fn tc_448_a_parameter_typed_at_an_object_type_is_still_refused() {
    let mut scene = ledger_scene();
    let key = scene.param("object_typed", &Ty::Object("Ledger"));
    let case = scene.finish().at(&key);
    let CheckedPackageV2ReadResult::Refused(refusal) = case.read() else {
        panic!("a parameter typed at an object type must be refused");
    };
    assert_eq!(
        (refusal.code, refusal.cause, refusal.path.clone()),
        (
            Code::InvalidSemanticGraph,
            None,
            Some(pointer(&format!(
                "/semantic_graph/nodes/{}/semantic_type",
                case.node
            )))
        ),
        "{refusal:?}"
    );
}

/// Trace: FR-347-AC-11
#[trace("TC-448", "FR-347-AC-11")]
#[test]
fn tc_448_a_union_payload_naming_an_object_type_is_still_refused() {
    let mut scene = ledger_scene();
    let object = scene.ty(&Ty::Object("Ledger"));
    let key = scene.union("Holder", &[("Hold", vec![object])]);
    scene.finish().at(&key).refuses(
        "object payload",
        Code::IllTyped,
        Cause::OperatorIneligible,
        "",
    );
}
