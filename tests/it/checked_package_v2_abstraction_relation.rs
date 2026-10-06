// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-225: the V2 reader admits QSpec FR-451's `correspondence`/
//! `abstraction_relation` body and refuses every other shape with the code,
//! cause and locus FR-346 names, in reader order.
//!
//! Every case builds its package in-repo over a domain package document built
//! here (`ConfigVersion` with the fields `version` and `generation`, the
//! operations `attemptUpdate(next)` and `rebase(from, to)`, a subtype
//! `ConfigVersionDraft` that only inherits them, and `Both`, which inherits one
//! field name and one operation name from two supertypes). Every expected
//! `node_id` is computed here from the body, through `sha256` over the
//! canonical bytes, and never through the reader.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_package, rebuild_source_map, refresh_identity,
    settle, sha256_hex, typed_node_id,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeKind, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    CorrespondenceForm,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

use CheckedPackageRefusalCause as Cause;
use CheckedPackageRefusalCode as Code;

const IDENTITY: &str = "acme/config";
const OTHER_IDENTITY: &str = "acme/other";
const CONFIG_VERSION: &str = "ix://acme/config/ConfigVersion";
const DRAFT: &str = "ix://acme/config/ConfigVersionDraft";
const LEFT: &str = "ix://acme/config/Left";
const RIGHT: &str = "ix://acme/config/Right";
const BOTH: &str = "ix://acme/config/Both";
const NATIVE_INTEGER: &str = "ix://quire/native/Integer";

const INTEGER: &str = "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f";
const TEXT: &str = "7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e";
const POPULATION_A: &str = "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a";
const POPULATION_B: &str = "2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b";
const FRAME: &str = "3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a";
const ANCHOR: &str = "3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b";
const FORMULA: &str = "4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a";

// The domain package document.

fn slot(type_ref: &str) -> Value {
    json!({"typeRef": type_ref,
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}})
}

fn domain_field(owner: &str, name: &str) -> Value {
    json!({
        "identity": format!("{owner}/{name}"), "name": name, "typeRef": NATIVE_INTEGER,
        "presence": "required", "nullable": false, "defaultKind": "none",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
    })
}

/// An operation `owner/name` returning an Integer over the named parameters.
fn domain_operation(owner: &str, name: &str, parameters: &[&str]) -> Value {
    let identity = format!("{owner}/{name}");
    let params = parameters
        .iter()
        .map(|parameter| {
            let mut declared = slot(NATIVE_INTEGER);
            declared["identity"] = json!(format!("{identity}/{parameter}"));
            declared["name"] = json!(parameter);
            declared
        })
        .collect::<Vec<_>>();
    json!({"identity": identity, "name": name, "params": params, "returns": slot(NATIVE_INTEGER)})
}

fn domain_type(
    node: &str,
    supertypes: &[&str],
    fields: Vec<Value>,
    operations: Vec<Value>,
) -> Value {
    json!({
        "identity": node, "displayName": node,
        "kind": {"module": IDENTITY, "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": fields, "operations": operations,
    })
}

fn domain_document() -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": IDENTITY, "version": "1.0.0"},
        "constructs": [{
            "kind": {"module": IDENTITY, "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [
            domain_type(
                CONFIG_VERSION,
                &[],
                vec![
                    domain_field(CONFIG_VERSION, "version"),
                    domain_field(CONFIG_VERSION, "generation"),
                ],
                vec![
                    domain_operation(CONFIG_VERSION, "attemptUpdate", &["next"]),
                    domain_operation(CONFIG_VERSION, "rebase", &["from", "to"]),
                ],
            ),
            domain_type(DRAFT, &[CONFIG_VERSION], vec![], vec![]),
            domain_type(
                LEFT,
                &[],
                vec![domain_field(LEFT, "shared")],
                vec![domain_operation(LEFT, "shared", &[])],
            ),
            domain_type(
                RIGHT,
                &[],
                vec![domain_field(RIGHT, "shared")],
                vec![domain_operation(RIGHT, "shared", &[])],
            ),
            domain_type(BOTH, &[LEFT, RIGHT], vec![], vec![]),
        ],
    })
}

// The package.

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn plain(
    digest: &str,
    tag: &str,
    form: &str,
    ty: &str,
    deps: &[&str],
    role: &str,
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(ty),
        "dependencies": deps.iter().map(|d| node_id(d)).collect::<Vec<_>>(),
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    })
}

/// The model declaration node key of `node` owned by the domain package
/// `identity`: the SHA-256 of its structural preimage.
fn model_key_in(identity: &str, node: &str) -> String {
    sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "model", "semantic_form": "object_type",
        "semantic_type": null, "declaration": null, "recursion": null,
        "body": empty(),
        "owner": {"kind": "model", "identity": identity, "node": node},
    })))
}

fn key(node: &str) -> String {
    model_key_in(IDENTITY, node)
}

fn config_version() -> String {
    key(CONFIG_VERSION)
}

/// The object type nodes of the package, in this order.
fn object_types() -> Vec<String> {
    [CONFIG_VERSION, DRAFT, LEFT, RIGHT, BOTH]
        .iter()
        .map(|node| key(node))
        .collect()
}

/// The package every case builds on: the lock selects the domain document,
/// and the graph holds the object type, population and scalar nodes the
/// relation nodes name.
fn base() -> Value {
    base_over(&domain_document())
}

/// [`base`] over a lock that selects `document`.
fn base_over(document: &Value) -> Value {
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": IDENTITY, "digest_domain": "sha256-jcs",
        "digest": sha256_hex(&canonical(document)),
    }]);
    let mut nodes = vec![
        plain(
            INTEGER,
            "scalar_type",
            "integer",
            INTEGER,
            &[],
            "type",
            empty(),
        ),
        plain(TEXT, "scalar_type", "text", TEXT, &[], "type", empty()),
        plain(
            POPULATION_A,
            "relation",
            "population",
            POPULATION_A,
            &[],
            "type",
            empty(),
        ),
        plain(
            POPULATION_B,
            "relation",
            "population",
            POPULATION_B,
            &[],
            "type",
            empty(),
        ),
    ];
    for (declared, model) in [CONFIG_VERSION, DRAFT, LEFT, RIGHT, BOTH]
        .into_iter()
        .zip(object_types())
    {
        let mut node = plain(&model, "model", "object_type", &model, &[], "type", empty());
        node["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": declared});
        nodes.push(node);
    }
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    settled(package)
}

fn settled(mut package: Value) -> Value {
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    package
}

fn nodes(package: &Value) -> &[Value] {
    package
        .pointer("/semantic_graph/nodes")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn position(value: &Value, digest: &str) -> usize {
    nodes(value)
        .iter()
        .position(|node| node["node_id"]["digest"] == digest)
        .unwrap_or_else(|| panic!("node {digest}"))
}

fn with_nodes(mut package: Value, added: Vec<Value>) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(added);
    settled(package)
}

/// Edits the node at `position` and re-derives the source map and identity;
/// the node's own `node_id` stays as the edit leaves it.
fn mutated(mut package: Value, position: usize, edit: impl FnOnce(&mut Value)) -> Value {
    edit(&mut package["semantic_graph"]["nodes"][position]);
    settled(package)
}

// The relation bodies.

fn object(ty: &str, rust_type: &[&str], fields: &[(&str, &str)]) -> Value {
    json!({
        "type": node_id(ty), "rust_type": rust_type,
        "fields": fields.iter().map(|(name, rust)| json!({"name": name, "rust_field": rust})).collect::<Vec<_>>(),
    })
}

fn population(population: &str, collection: &[&str]) -> Value {
    json!({"population": node_id(population), "collection": collection})
}

fn frame(
    context: &str,
    operation: &str,
    function: &[&str],
    receiver: &str,
    parameters: &[(&str, &str)],
) -> Value {
    json!({
        "context": node_id(context), "operation": operation, "function": function,
        "receiver": receiver,
        "parameters": parameters.iter().map(|(name, rust)| json!({"name": name, "rust_parameter": rust})).collect::<Vec<_>>(),
    })
}

fn body(objects: Vec<Value>, populations: Vec<Value>, frames: Vec<Value>) -> Value {
    json!({
        "term": "abstraction_relation",
        "objects": objects, "populations": populations, "frames": frames,
    })
}

/// `body` with every array in its canonical order.
fn ordered(mut body: Value) -> Value {
    let digest = |value: &Value, member: &str| {
        value[member]["digest"]
            .as_str()
            .expect("a node key")
            .to_owned()
    };
    let name = |value: &Value| value["name"].as_str().expect("a name").to_owned();
    for entry in body["objects"].as_array_mut().expect("objects").iter_mut() {
        entry["fields"]
            .as_array_mut()
            .expect("fields")
            .sort_by_key(name);
    }
    for entry in body["frames"].as_array_mut().expect("frames").iter_mut() {
        entry["parameters"]
            .as_array_mut()
            .expect("parameters")
            .sort_by_key(name);
    }
    body["objects"]
        .as_array_mut()
        .expect("objects")
        .sort_by_key(|entry| digest(entry, "type"));
    body["populations"]
        .as_array_mut()
        .expect("populations")
        .sort_by_key(|entry| digest(entry, "population"));
    body["frames"]
        .as_array_mut()
        .expect("frames")
        .sort_by_key(|entry| {
            (
                digest(entry, "context"),
                entry["operation"].as_str().expect("operation").to_owned(),
            )
        });
    body
}

/// AC-1's body: one object, no population and one frame.
fn minimal() -> Value {
    ordered(body(
        vec![object(
            &config_version(),
            &["config_store", "ConfigVersion"],
            &[("version", "version")],
        )],
        vec![],
        vec![frame(
            &config_version(),
            "attemptUpdate",
            &["config_store", "attempt_update"],
            "self",
            &[("next", "next")],
        )],
    ))
}

/// A body with an entry of every array.
fn rich() -> Value {
    ordered(body(
        vec![object(
            &config_version(),
            &["config_store", "ConfigVersion"],
            &[("version", "version"), ("generation", "generation")],
        )],
        vec![population(POPULATION_A, &["config_store", "versions"])],
        vec![frame(
            &config_version(),
            "rebase",
            &["config_store", "rebase"],
            "self",
            &[("from", "from"), ("to", "to")],
        )],
    ))
}

/// The node key of a relation body: the SHA-256 of the canonical bytes of
/// `{version, body}`, computed here and not by the reader.
fn digest_of(body: &Value) -> String {
    sha256_hex(&canonical(
        &json!({"version": "quire.abstraction-relation-node/v1", "body": body}),
    ))
}

/// Every `type`, `population` and `context` target of `body`, unique and
/// digest-ascending.
fn targets(body: &Value) -> Vec<String> {
    let mut keys = BTreeSet::new();
    for (array, member) in [
        ("objects", "type"),
        ("populations", "population"),
        ("frames", "context"),
    ] {
        for entry in body[array].as_array().into_iter().flatten() {
            if let Some(digest) = entry[member]["digest"].as_str() {
                keys.insert(digest.to_owned());
            }
        }
    }
    keys.into_iter().collect()
}

fn relation_node(body: &Value) -> Value {
    let id = digest_of(body);
    let targets = targets(body);
    let targets = targets.iter().map(String::as_str).collect::<Vec<_>>();
    plain(
        &id,
        "correspondence",
        "abstraction_relation",
        &id,
        &targets,
        "declaration",
        body.clone(),
    )
}

/// `base()` holding one relation node per body.
fn relations(bodies: &[Value]) -> Value {
    with_nodes(base(), bodies.iter().map(relation_node).collect())
}

/// The relation node of `body` in `package`.
fn at(package: &Value, body: &Value) -> usize {
    position(package, &digest_of(body))
}

/// Replaces the body of the relation node at `position` and re-derives its
/// key, type and dependencies from the new body.
fn rebodied(package: Value, position: usize, new_body: Value) -> Value {
    let fresh = relation_node(&new_body);
    mutated(package, position, move |node| *node = fresh)
}

// Reading.

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    read_over(value, &domain_document())
}

/// Reads `value` with `document` supplied as the selected domain package.
fn read_over(value: &Value, document: &Value) -> CheckedPackageV2ReadResult {
    let mut evidence = evidence_for(value);
    evidence.insert_domain_package_document(sha256_hex(&canonical(document)), canonical(document));
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn admitted(case: &str, value: &Value) -> CheckedPackageV2 {
    match read(value) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

fn refused(case: &str, value: &Value) -> CheckedPackageRefusal {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{case}: expected a refusal, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause, pointer and (when given) locus digest.
fn expect(
    case: &str,
    value: &Value,
    code: Code,
    cause: Option<Cause>,
    path: &str,
    locus: Option<&str>,
) {
    let refusal = refused(case, value);
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{case}: {refusal:?}");
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
    if let Some(locus) = locus {
        assert_eq!(
            refusal.locus.as_ref().map(|id| &*id.digest),
            Some(locus),
            "{case}: {refusal:?}"
        );
    }
}

/// `/semantic_graph/nodes/{n}` followed by `suffix`.
fn at_node(position: usize, suffix: &str) -> String {
    format!("/semantic_graph/nodes/{position}{suffix}")
}

// AC-1.

/// Tracing: TC-225
/// ACs: FR-346-AC-1
#[trace("TC-225", "FR-346-AC-1")]
#[test]
fn tc_225_the_minimal_abstraction_relation_body_admits_with_its_computed_key() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let package = admitted("the minimal body", &value);
    let position = at(&value, &minimal);
    assert_eq!(
        package.node_kinds()[position],
        CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
    );
    let node = &package.graph().nodes[position];
    assert_eq!(node.node_id, typed_node_id(&digest_of(&minimal)));
    assert_eq!(node.semantic_type, node.node_id);
    assert_eq!(node.body, minimal);
    assert_eq!(
        node.dependencies,
        vec![typed_node_id(&config_version())],
        "one object and one frame name one type"
    );
}

/// The relation's object-field entry reads the selected type's field table.
/// `Draft` inherits `ConfigVersion`'s two fields and declares `extra`, so a
/// full read of the same selected document costs one more unit when the entry
/// names `Draft`.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_an_abstraction_field_entry_charges_its_effective_table_end_to_end() {
    let mut document = domain_document();
    document["types"][1]["fields"] = json!([domain_field(DRAFT, "extra")]);
    let smallest = |model: &str| {
        let body = ordered(body(
            vec![object(
                model,
                &["config_store", "Version"],
                &[("version", "version")],
            )],
            vec![],
            vec![],
        ));
        let value = with_nodes(base_over(&document), vec![relation_node(&body)]);
        let mut evidence = evidence_for(&value);
        evidence.insert_domain_package_document(
            sha256_hex(&canonical(&document)),
            canonical(&document),
        );
        let read = |work| {
            let mut limits = CheckedPackageReadLimits::bounded();
            limits.work = work;
            CheckedPackageV2::read(&canonical(&value), limits, &evidence)
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
    assert_eq!(smallest(&key(DRAFT)), smallest(&config_version()) + 1);
}

/// Tracing: TC-225
/// ACs: FR-346-AC-1
#[trace("TC-225", "FR-346-AC-1")]
#[test]
fn tc_225_empty_tuple_field_and_raw_identifier_bodies_admit() {
    for (case, body) in [
        ("all three arrays empty", body(vec![], vec![], vec![])),
        ("a body with an entry of every array", rich()),
        (
            "a tuple-field index and a raw identifier",
            ordered(body(
                vec![object(
                    &config_version(),
                    &["r#type"],
                    &[("version", "0"), ("generation", "r#type")],
                )],
                vec![population(POPULATION_A, &["r#match", "collection"])],
                vec![frame(
                    &config_version(),
                    "attemptUpdate",
                    &["r#fn"],
                    "r#type",
                    &[("next", "r#next")],
                )],
            )),
        ),
        (
            "a multi-digit tuple-field index",
            ordered(body(
                vec![object(&config_version(), &["pair"], &[("version", "10")])],
                vec![],
                vec![],
            )),
        ),
    ] {
        let value = relations(std::slice::from_ref(&body));
        let package = admitted(case, &value);
        assert_eq!(
            package.graph().nodes[at(&value, &body)].node_id,
            typed_node_id(&digest_of(&body)),
            "{case}"
        );
    }
}

/// Tracing: TC-225
/// ACs: FR-346-AC-1
#[trace("TC-225", "FR-346-AC-1")]
#[test]
fn tc_225_an_unrecognized_correspondence_form_still_refuses_at_semantic_form() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let position = at(&value, &minimal);
    let value = mutated(value, position, |node| {
        node["semantic_form"] = json!("abstraction_relations");
    });
    expect(
        "an unrecognized form",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &at_node(position, "/semantic_form"),
        None,
    );
}

// AC-2.

fn application(operator: &str, identity: &str) -> Value {
    json!({
        "term": "application", "operator": operator,
        "operation": {"identity": identity, "laws": [], "mode": null, "member": null, "leaves": []},
        "result_type": node_id(INTEGER),
        "arguments": [{"term": "reference", "target": node_id(INTEGER)}],
    })
}

/// Tracing: TC-225
/// ACs: FR-346-AC-2
#[trace("TC-225", "FR-346-AC-2")]
#[test]
fn tc_225_a_body_off_its_closed_shape_refuses_without_a_cause() {
    type Edit = Box<dyn Fn(&mut Value)>;
    let rich = rich();
    let value = relations(std::slice::from_ref(&rich));
    let n = at(&value, &rich);
    let cases: Vec<(&str, Edit, &str)> = vec![
        (
            "a body missing frames",
            Box::new(|node| {
                node["body"].as_object_mut().expect("body").remove("frames");
            }),
            "/body",
        ),
        (
            "a body with a fourth member",
            Box::new(|node| node["body"]["extra"] = json!([])),
            "/body",
        ),
        (
            "a member of the wrong type",
            Box::new(|node| node["body"]["objects"] = json!(5)),
            "/body",
        ),
        (
            "a term of aggregate",
            Box::new(|node| node["body"]["term"] = json!("aggregate")),
            "/body",
        ),
        (
            "an objects entry missing fields",
            Box::new(|node| {
                node["body"]["objects"][0]
                    .as_object_mut()
                    .expect("entry")
                    .remove("fields");
            }),
            "/body/objects/0",
        ),
        (
            "a type that is not a node key",
            Box::new(|node| node["body"]["objects"][0]["type"] = json!("a name")),
            "/body/objects/0",
        ),
        (
            "an empty rust_type",
            Box::new(|node| node["body"]["objects"][0]["rust_type"] = json!([])),
            "/body/objects/0",
        ),
        (
            "an objects entry with an extra member",
            Box::new(|node| node["body"]["objects"][0]["extra"] = json!(1)),
            "/body/objects/0",
        ),
        (
            "a fields entry missing rust_field",
            Box::new(|node| {
                node["body"]["objects"][0]["fields"][0]
                    .as_object_mut()
                    .expect("entry")
                    .remove("rust_field");
            }),
            "/body/objects/0/fields/0",
        ),
        (
            "a fields name that is not an identifier",
            Box::new(|node| node["body"]["objects"][0]["fields"][0]["name"] = json!("a b")),
            "/body/objects/0/fields/0",
        ),
        (
            "a populations entry missing collection",
            Box::new(|node| {
                node["body"]["populations"][0]
                    .as_object_mut()
                    .expect("entry")
                    .remove("collection");
            }),
            "/body/populations/0",
        ),
        (
            "a frame entry missing receiver",
            Box::new(|node| {
                node["body"]["frames"][0]
                    .as_object_mut()
                    .expect("entry")
                    .remove("receiver");
            }),
            "/body/frames/0",
        ),
        (
            "a function that is an empty array",
            Box::new(|node| node["body"]["frames"][0]["function"] = json!([])),
            "/body/frames/0",
        ),
        (
            "an operation that is not an identifier",
            Box::new(|node| node["body"]["frames"][0]["operation"] = json!("re base")),
            "/body/frames/0",
        ),
        (
            "a parameters entry missing rust_parameter",
            Box::new(|node| {
                node["body"]["frames"][0]["parameters"][1]
                    .as_object_mut()
                    .expect("entry")
                    .remove("rust_parameter");
            }),
            "/body/frames/0/parameters/1",
        ),
    ];
    for (case, edit, suffix) in cases {
        let value = mutated(value.clone(), n, edit);
        expect(
            case,
            &value,
            Code::InvalidSemanticGraph,
            None,
            &at_node(n, suffix),
            Some(&digest_of(&rich)),
        );
    }
}

/// Tracing: TC-225
/// ACs: FR-346-AC-2
#[trace("TC-225", "FR-346-AC-2")]
#[test]
fn tc_225_an_occurrence_role_or_another_form_off_the_relation_refuses() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let n = at(&value, &minimal);
    let anchored = mutated(value.clone(), n, |node| {
        node["occurrences"] =
            json!([{"role": "declaration", "ordinal": 0}, {"role": "anchor", "ordinal": 0}]);
    });
    expect(
        "an occurrence of role anchor",
        &anchored,
        Code::InvalidSemanticGraph,
        None,
        &at_node(n, "/occurrences/1/role"),
        None,
    );
    for form in ["model_correspondence", "source_locus"] {
        let carried = mutated(value.clone(), n, |node| node["semantic_form"] = json!(form));
        expect(
            "another form carrying the body",
            &carried,
            Code::InvalidSemanticGraph,
            None,
            &at_node(n, "/body"),
            None,
        );
    }
}

/// A relation node whose body root is `root`, keyed and joined as an
/// application node, and the package holding it.
fn rooted_in(root: Value) -> (Value, usize) {
    let id = digest_of(&root);
    let node = plain(
        &id,
        "correspondence",
        "abstraction_relation",
        INTEGER,
        &[],
        "declaration",
        root,
    );
    let mut value = with_nodes(base(), vec![node]);
    settle(&mut value);
    let n = nodes(&value).len() - 1;
    (value, n)
}

/// Tracing: TC-225
/// ACs: FR-346-AC-2
#[trace("TC-225", "FR-346-AC-2")]
#[test]
fn tc_225_an_application_at_the_body_root_refuses_as_operator_ineligible() {
    let state_clause = json!({
        "term": "application", "operator": "state_clause",
        "operation": {"identity": "quire.op.state.clause", "laws": [], "mode": null,
            "member": {"kind": "state_clause", "clause": "invariant"}, "leaves": []},
        "result_type": node_id(INTEGER),
        "arguments": [
            {"term": "aggregate", "members": [{"term": "reference", "target": node_id(INTEGER)}]},
            {"term": "reference", "target": node_id(INTEGER)},
            {"term": "reference", "target": node_id(INTEGER)},
        ],
    });
    // The abstraction step's shape check and the temporal step refuse at the
    // node; the state step, as for every node it places a clause application
    // at (FR-040-AC-10), refuses at the node's body.
    for (case, root, suffix) in [
        (
            "an ordinary class",
            application("call", "quire.op.function.call"),
            "",
        ),
        (
            "a temporal_formula class",
            application("temporal_formula", "quire.op.temporal.holds"),
            "",
        ),
        ("the state clause class", state_clause, "/body"),
    ] {
        let (value, n) = rooted_in(root);
        expect(
            case,
            &value,
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            &at_node(n, suffix),
            Some(nodes(&value)[n]["node_id"]["digest"].as_str().expect("key")),
        );
    }
}

/// Tracing: TC-225
/// ACs: FR-346-AC-2
#[trace("TC-225", "FR-346-AC-2")]
#[test]
fn tc_225_an_application_as_a_member_value_refuses_at_strict_wire_validation() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let n = at(&value, &minimal);
    let with_type = |term: Value| {
        mutated(value.clone(), n, move |node| {
            node["body"]["objects"][0]["type"] = term;
        })
    };
    expect(
        "a non-case application",
        &with_type(application("call", "quire.op.function.call")),
        Code::MalformedWire,
        None,
        &at_node(n, "/body/objects/0/type"),
        None,
    );
    expect(
        "a case application",
        &with_type(application("case", "quire.op.expression.case")),
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &at_node(n, "/body/objects/0/type/operator"),
        Some(&digest_of(&minimal)),
    );
    // The member that holds it is any member, not only a node key.
    let in_frame = mutated(value.clone(), n, |node| {
        node["body"]["frames"][0]["function"] =
            json!([application("call", "quire.op.function.call")]);
    });
    expect(
        "an application inside a function path",
        &in_frame,
        Code::MalformedWire,
        None,
        &at_node(n, "/body/frames/0/function/0"),
        None,
    );
}

/// A member nested to within a few levels of the strict parse's recursion limit
/// (110 arrays, 117 JSON levels of the 127 the parse reads), far past the nesting
/// of any in-grammar body, is scanned and refused as a body of the wrong shape on
/// a 256 KiB stack, in a debug build too.
///
/// Tracing: TC-225
/// ACs: FR-346-AC-2
#[trace("TC-225", "FR-346-AC-2")]
#[test]
fn tc_225_a_deeply_nested_member_refuses_as_the_wrong_shape() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let n = at(&value, &minimal);
    let mut nested = json!("leaf");
    for _ in 0..110 {
        nested = json!([nested]);
    }
    let deep = mutated(value, n, |node| {
        node["body"]["objects"][0]["rust_type"] = nested;
    });
    let document = domain_document();
    let mut evidence = evidence_for(&deep);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(&document)), canonical(&document));
    let bytes = canonical(&deep);
    let limits = CheckedPackageReadLimits::bounded();
    // The read runs on a stack far smaller than the frames a native recursion
    // over a body this deep needs, so a walk that recurses aborts the process.
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(
            move || match CheckedPackageV2::read(&bytes, limits, &evidence) {
                CheckedPackageV2ReadResult::Refused(refusal) => {
                    assert_eq!(refusal.code, Code::InvalidSemanticGraph);
                    assert_eq!(
                        refusal.path.as_ref().map(|path| path.as_str()),
                        Some(at_node(n, "/body/objects/0").as_str())
                    );
                }
                other => panic!("expected a refusal, read {other:?}"),
            },
        )
        .expect("spawn")
        .join()
        .expect("the read ran to completion");
}

/// A member-defective relation body whose node key sorts below (`lower`) or
/// above the node key `than`.
fn member_defect_keyed(than: &str, lower: bool) -> Value {
    (0..256)
        .map(|index| member_defect(&format!("missing{index}")))
        .find(|candidate| (digest_of(candidate).as_str() < than) == lower)
        .expect("a candidate on that side of the key")
}

/// The step that refuses a root application is told apart by node-id order:
/// the temporal step runs before the abstraction step over every node, so a
/// temporal-class root is reported ahead of a member defect of a lower node
/// key; an ordinary-class root is the abstraction step's own, and is reported
/// in node-key order with that defect.
///
/// Tracing: TC-225
/// ACs: FR-346-AC-2
#[trace("TC-225", "FR-346-AC-2")]
#[test]
fn tc_225_a_root_application_is_refused_by_the_step_that_places_it() {
    let temporal = application("temporal_formula", "quire.op.temporal.holds");
    let ordinary = application("call", "quire.op.function.call");
    let root_refusal = |case: &str, value: &Value, rooted_at: usize, root_key: &str| {
        expect(
            case,
            value,
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            &at_node(rooted_at, ""),
            Some(root_key),
        );
    };
    // The temporal class: the root is reported though a member defect sits on
    // a node of a lower key, which the abstraction step would reach first.
    let (rooted, rooted_at) = rooted_in(temporal);
    let root_key = nodes(&rooted)[rooted_at]["node_id"]["digest"]
        .as_str()
        .expect("key")
        .to_owned();
    let defect = member_defect_keyed(&root_key, true);
    let value = with_nodes(rooted, vec![relation_node(&defect)]);
    root_refusal(
        "a temporal root and a lower defect",
        &value,
        rooted_at,
        &root_key,
    );
    // The ordinary class: the abstraction step reports the two nodes in key
    // order, the lower first.
    let (rooted, rooted_at) = rooted_in(ordinary);
    let root_key = nodes(&rooted)[rooted_at]["node_id"]["digest"]
        .as_str()
        .expect("key")
        .to_owned();
    let below = member_defect_keyed(&root_key, true);
    let value = with_nodes(rooted.clone(), vec![relation_node(&below)]);
    assert_eq!(
        refusal_path("an ordinary root and a lower defect", &value),
        at_node(at(&value, &below), "/body/objects/0/fields/0")
    );
    let above = member_defect_keyed(&root_key, false);
    let value = with_nodes(rooted, vec![relation_node(&above)]);
    root_refusal(
        "an ordinary root and a higher defect",
        &value,
        rooted_at,
        &root_key,
    );
}

/// A node key outside the node-identity domain, or whose digest is not
/// lowercase hex, is `digest_domain_mismatch` at the key, as every other node
/// reference the reader reads; a value that is no node key is an entry of the
/// wrong shape.
///
/// Tracing: TC-225
/// ACs: FR-346-AC-2
#[trace("TC-225", "FR-346-AC-2")]
#[test]
fn tc_225_a_node_key_outside_the_node_domain_refuses_as_a_domain_mismatch() {
    let rich = rich();
    let value = relations(std::slice::from_ref(&rich));
    let n = at(&value, &rich);
    let keys = [
        (
            "another digest domain",
            json!({"domain": "quire.other-node/v1", "digest": config_version()}),
        ),
        (
            "a digest that is not hex",
            json!({"domain": "quire.checked-semantic-node/v1", "digest": "z".repeat(64)}),
        ),
        (
            "an uppercase digest",
            json!({"domain": "quire.checked-semantic-node/v1", "digest": config_version().to_uppercase()}),
        ),
        (
            "a short digest",
            json!({"domain": "quire.checked-semantic-node/v1", "digest": "ab"}),
        ),
    ];
    for (case, bad) in keys {
        for (member_at, set) in [
            (
                "/body/objects/0/type",
                (|node: &mut Value, key: Value| node["body"]["objects"][0]["type"] = key)
                    as fn(&mut Value, Value),
            ),
            ("/body/populations/0/population", |node, key| {
                node["body"]["populations"][0]["population"] = key;
            }),
            ("/body/frames/0/context", |node, key| {
                node["body"]["frames"][0]["context"] = key;
            }),
        ] {
            let case = format!("{case} at {member_at}");
            let bad = bad.clone();
            let value = mutated(value.clone(), n, |node| set(node, bad));
            expect(
                &case,
                &value,
                Code::DigestDomainMismatch,
                None,
                &at_node(n, member_at),
                None,
            );
        }
    }
}

// AC-3.

fn state_frame(context: &str) -> Value {
    plain(
        FRAME,
        "state",
        "frame",
        context,
        &[context],
        "generated",
        json!({"term": "frame", "modifies": [], "creates": [], "deletes": []}),
    )
}

fn operation_anchor(context: &str, operation: &str) -> Value {
    let mut dependencies = [context.to_owned(), FRAME.to_owned()];
    dependencies.sort();
    let literal =
        json!({"term": "literal", "type": node_id(TEXT), "value_kind": "text", "value": operation});
    plain(
        ANCHOR,
        "state",
        "operation_anchor",
        context,
        &[&dependencies[0], &dependencies[1]],
        "anchor",
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "context", "value": {"term": "reference", "target": node_id(context)}},
            {"term": "binding", "name": "operation", "value": literal},
            {"term": "binding", "name": "frame", "value": {"term": "reference", "target": node_id(FRAME)}},
        ]}),
    )
}

/// Tracing: TC-225
/// ACs: FR-346-AC-3
#[trace("TC-225", "FR-346-AC-3")]
#[test]
fn tc_225_a_frame_entry_binds_its_pair_with_or_without_a_frame_node() {
    let minimal = minimal();
    let without = relations(std::slice::from_ref(&minimal));
    assert!(
        nodes(&without)
            .iter()
            .all(|node| node["semantic_form"] != "frame"),
        "the package holds no state/frame node"
    );
    admitted("no state/frame node for the pair", &without);
    let context = config_version();
    let with = with_nodes(
        relations(std::slice::from_ref(&minimal)),
        vec![
            state_frame(&context),
            operation_anchor(&context, "attemptUpdate"),
        ],
    );
    let package = admitted("a state/frame node and its anchor for the pair", &with);
    // The pairs are read back from the admitted graph: the relation node's
    // frame entry and the anchor's body.
    let pair_of = |package: &CheckedPackageV2| {
        let graph = &package.graph().nodes;
        let entry = graph
            .iter()
            .find(|node| node.semantic_form.as_ref() == "abstraction_relation")
            .map(|node| &node.body["frames"][0])
            .expect("the relation node");
        let anchor = graph
            .iter()
            .find(|node| node.semantic_form.as_ref() == "operation_anchor")
            .map(|node| &node.body["members"])
            .expect("the anchor node");
        (
            (entry["context"].clone(), entry["operation"].clone()),
            (
                anchor[0]["value"]["target"].clone(),
                anchor[1]["value"]["value"].clone(),
            ),
        )
    };
    let (entry_pair, anchor_pair) = pair_of(&package);
    assert_eq!(entry_pair, anchor_pair);
    // The comparison can fail: an anchor for another operation does not match
    // the entry (and the package still admits, as the entry binds its pair
    // whether or not a frame node holds it).
    let other = with_nodes(
        relations(std::slice::from_ref(&minimal)),
        vec![state_frame(&context), operation_anchor(&context, "rebase")],
    );
    let other = admitted("an anchor for another operation", &other);
    let (entry_pair, anchor_pair) = pair_of(&other);
    assert_ne!(entry_pair, anchor_pair);
}

// AC-4.

/// Tracing: TC-225
/// ACs: FR-346-AC-4
#[trace("TC-225", "FR-346-AC-4")]
#[test]
fn tc_225_identity_members_that_do_not_hold_refuse_at_the_node() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let n = at(&value, &minimal);
    let own = digest_of(&minimal);
    type Edit = Box<dyn Fn(&mut Value)>;
    let cases: Vec<(&str, Edit)> = vec![
        (
            "a stale node_id",
            Box::new(|node| node["node_id"] = node_id(&"9".repeat(64))),
        ),
        (
            "a semantic_type other than the node",
            Box::new(|node| node["semantic_type"] = node_id(INTEGER)),
        ),
        (
            "dependencies omitting a body target",
            Box::new(|node| node["dependencies"] = json!([])),
        ),
        (
            "dependencies holding a target the body does not",
            Box::new(|node| {
                let mut dependencies = vec![node_id(&config_version()), node_id(INTEGER)];
                dependencies.sort_by_key(|id| id["digest"].as_str().map(str::to_owned));
                node["dependencies"] = Value::Array(dependencies);
            }),
        ),
    ];
    for (case, edit) in cases {
        let value = mutated(value.clone(), n, edit);
        // The node key may have changed; the locus is whatever the node now
        // carries.
        let locus = nodes(&value)[n]["node_id"]["digest"]
            .as_str()
            .expect("key")
            .to_owned();
        expect(
            case,
            &value,
            Code::InvalidSemanticGraph,
            None,
            &at_node(n, ""),
            Some(&locus),
        );
    }
    // A stale key is the abstraction step's, never `stale-node-key`.
    let stale = mutated(value, n, |node| node["node_id"] = node_id(&"9".repeat(64)));
    let refusal = refused("a stale node_id", &stale);
    assert_ne!(refusal.cause, Some(Cause::StaleNodeKey));
    assert_ne!(own, "9".repeat(64));
}

/// Tracing: TC-225
/// ACs: FR-346-AC-4
#[trace("TC-225", "FR-346-AC-4")]
#[test]
fn tc_225_an_array_out_of_its_canonical_order_refuses_at_the_array() {
    let version_object = |ty: &str| object(ty, &["t"], &[("version", "version")]);
    let types = {
        let mut types = [config_version(), key(DRAFT)];
        types.sort();
        types
    };
    let two_objects = body(
        vec![version_object(&types[1]), version_object(&types[0])],
        vec![],
        vec![],
    );
    let populations = {
        let mut keys = [POPULATION_A, POPULATION_B];
        keys.sort();
        body(
            vec![],
            vec![population(keys[1], &["b"]), population(keys[0], &["a"])],
            vec![],
        )
    };
    let frames = {
        let rebase = frame(
            &config_version(),
            "rebase",
            &["f"],
            "self",
            &[("from", "a"), ("to", "b")],
        );
        let attempt = frame(
            &config_version(),
            "attemptUpdate",
            &["g"],
            "self",
            &[("next", "n")],
        );
        body(vec![], vec![], vec![rebase, attempt])
    };
    let fields = body(
        vec![object(
            &config_version(),
            &["t"],
            &[("version", "version"), ("generation", "generation")],
        )],
        vec![],
        vec![],
    );
    let parameters = body(
        vec![],
        vec![],
        vec![frame(
            &config_version(),
            "rebase",
            &["f"],
            "self",
            &[("to", "b"), ("from", "a")],
        )],
    );
    for (case, body, suffix) in [
        (
            "objects out of type digest order",
            two_objects,
            "/body/objects",
        ),
        ("populations out of order", populations, "/body/populations"),
        (
            "two frames of one context in descending operation order",
            frames,
            "/body/frames",
        ),
        ("fields out of name order", fields, "/body/objects/0/fields"),
        (
            "parameters out of name order",
            parameters,
            "/body/frames/0/parameters",
        ),
    ] {
        let value = relations(std::slice::from_ref(&body));
        expect(
            case,
            &value,
            Code::InvalidSemanticGraph,
            None,
            &at_node(at(&value, &body), suffix),
            Some(&digest_of(&body)),
        );
    }
}

// AC-5.

/// Tracing: TC-225
/// ACs: FR-346-AC-5, FR-038-AC-155
#[trace("TC-225", "FR-346-AC-5", "FR-038-AC-155")]
#[test]
fn tc_225_a_target_of_the_wrong_kind_or_unselected_owner_refuses_at_the_target() {
    let unselected = model_key_in(OTHER_IDENTITY, CONFIG_VERSION);
    let in_object = |ty: &str| body(vec![object(ty, &["t"], &[])], vec![], vec![]);
    let in_population = |ty: &str| body(vec![], vec![population(ty, &["p"])], vec![]);
    let in_frame = |context: &str| {
        body(
            vec![],
            vec![],
            vec![frame(
                context,
                "attemptUpdate",
                &["f"],
                "self",
                &[("next", "n")],
            )],
        )
    };
    let malformed = (Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let cases = [
        (
            "a type naming a population",
            in_object(POPULATION_A),
            POPULATION_A,
            "/body/objects/0",
            malformed,
        ),
        (
            "a population naming an object type",
            in_population(&config_version()),
            &config_version(),
            "/body/populations/0",
            malformed,
        ),
        (
            "a context naming a state/frame node",
            in_frame(FRAME),
            FRAME,
            "/body/frames/0",
            malformed,
        ),
    ];
    for (case, body, locus, suffix, (code, cause)) in cases {
        let value = with_nodes(
            relations(std::slice::from_ref(&body)),
            vec![state_frame(&config_version())],
        );
        expect(
            case,
            &value,
            code,
            Some(cause),
            &at_node(at(&value, &body), suffix),
            Some(locus),
        );
    }
    for (case, body) in [
        ("unselected frame context", in_frame(&unselected)),
        ("unselected object type", in_object(&unselected)),
    ] {
        let mut node = plain(
            &unselected,
            "model",
            "object_type",
            &unselected,
            &[],
            "type",
            empty(),
        );
        node["owner"] = json!({
            "kind": "model", "identity": OTHER_IDENTITY, "node": CONFIG_VERSION,
        });
        let value = with_nodes(
            relations(std::slice::from_ref(&body)),
            vec![state_frame(&config_version()), node],
        );
        let at = position(&value, &unselected);
        expect(
            case,
            &value,
            Code::MissingDeclaration,
            Some(Cause::MissingSelection),
            &at_node(at, "/node_id"),
            Some(&unselected),
        );
    }
}

// AC-6.

/// Tracing: TC-225
/// ACs: FR-346-AC-6
#[trace("TC-225", "FR-346-AC-6")]
#[test]
fn tc_225_a_field_binding_or_rust_spelling_off_its_syntax_refuses_at_the_entry() {
    let cv = config_version();
    let malformed = (Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let attempt = |function: &[&str], receiver: &str| {
        frame(&cv, "attemptUpdate", function, receiver, &[("next", "n")])
    };
    let cases = [
        (
            "a fields name the type does not expose",
            body(
                vec![object(&cv, &["t"], &[("missing", "m")])],
                vec![],
                vec![],
            ),
            "/body/objects/0/fields/0",
            malformed,
        ),
        (
            "a rust_field that is not a name",
            body(
                vec![object(&cv, &["t"], &[("version", "not a name")])],
                vec![],
                vec![],
            ),
            "/body/objects/0/fields/0",
            malformed,
        ),
        (
            "a rust_field with a leading zero",
            body(
                vec![object(&cv, &["t"], &[("version", "01")])],
                vec![],
                vec![],
            ),
            "/body/objects/0/fields/0",
            malformed,
        ),
        (
            "a rust_type segment that starts with a digit",
            body(vec![object(&cv, &["9lives"], &[])], vec![], vec![]),
            "/body/objects/0",
            malformed,
        ),
        (
            "a collection segment that is a keyword",
            body(vec![], vec![population(POPULATION_A, &["fn"])], vec![]),
            "/body/populations/0",
            malformed,
        ),
        (
            "a function segment that is crate",
            body(vec![], vec![], vec![attempt(&["crate", "f"], "self")]),
            "/body/frames/0",
            malformed,
        ),
        (
            "a receiver that is a path",
            body(vec![], vec![], vec![attempt(&["f"], "a::b")]),
            "/body/frames/0",
            malformed,
        ),
        (
            "a rust_parameter that is a keyword",
            body(
                vec![],
                vec![],
                vec![frame(
                    &cv,
                    "attemptUpdate",
                    &["f"],
                    "self",
                    &[("next", "match")],
                )],
            ),
            "/body/frames/0/parameters/0",
            malformed,
        ),
        (
            "two fields entries named version bound to different Rust fields",
            body(
                vec![object(&cv, &["t"], &[("version", "a"), ("version", "b")])],
                vec![],
                vec![],
            ),
            "/body/objects/0/fields/1",
            (Code::InvalidModelBinding, Cause::ConflictingBinding),
        ),
        (
            "a fields name two supertypes both expose",
            body(
                vec![object(&key(BOTH), &["t"], &[("shared", "s")])],
                vec![],
                vec![],
            ),
            "/body/objects/0/fields/0",
            (Code::AmbiguousDeclaration, Cause::AmbiguousName),
        ),
    ];
    for (case, body, suffix, (code, cause)) in cases {
        let body = ordered(body);
        let value = relations(std::slice::from_ref(&body));
        let locus = body["objects"]
            .get(0)
            .map(|entry| &entry["type"])
            .or_else(|| body["frames"].get(0).map(|entry| &entry["context"]))
            .or_else(|| body["populations"].get(0).map(|entry| &entry["population"]))
            .and_then(|id| id["digest"].as_str())
            .expect("a key")
            .to_owned();
        expect(
            case,
            &value,
            code,
            Some(cause),
            &at_node(at(&value, &body), suffix),
            Some(&locus),
        );
    }
}

// AC-7.

/// Tracing: TC-225
/// ACs: FR-346-AC-7
#[trace("TC-225", "FR-346-AC-7")]
#[test]
fn tc_225_a_frame_operation_resolves_as_an_anchor_does_and_binds_its_parameters() {
    let cv = config_version();
    let malformed = (Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let missing = (Code::MissingDeclaration, Cause::MissingName);
    let in_frame = |context: &str, operation: &str, receiver: &str, parameters: &[(&str, &str)]| {
        body(
            vec![],
            vec![],
            vec![frame(context, operation, &["f"], receiver, parameters)],
        )
    };
    let cases = [
        (
            "an undeclared operation",
            in_frame(&cv, "noSuchOperation", "self", &[]),
            &cv,
            "/body/frames/0",
            missing,
        ),
        (
            "the name of a field",
            in_frame(&cv, "version", "self", &[]),
            &cv,
            "/body/frames/0",
            missing,
        ),
        (
            "an operation two supertypes both carry",
            in_frame(&key(BOTH), "shared", "self", &[]),
            &key(BOTH),
            "/body/frames/0",
            (Code::AmbiguousDeclaration, Cause::AmbiguousName),
        ),
        (
            "an operation the context only inherits",
            in_frame(&key(DRAFT), "attemptUpdate", "self", &[("next", "n")]),
            &key(DRAFT),
            "/body/frames/0",
            malformed,
        ),
        (
            "a parameters list missing next",
            in_frame(&cv, "attemptUpdate", "self", &[]),
            &cv,
            "/body/frames/0",
            malformed,
        ),
        (
            "a parameters list naming an undeclared parameter",
            in_frame(
                &cv,
                "attemptUpdate",
                "self",
                &[("next", "n"), ("other", "o")],
            ),
            &cv,
            "/body/frames/0",
            malformed,
        ),
        (
            "two rust_parameter values that are equal",
            in_frame(&cv, "rebase", "self", &[("from", "a"), ("to", "a")]),
            &cv,
            "/body/frames/0/parameters/1",
            malformed,
        ),
        (
            "a rust_parameter equal to the receiver",
            in_frame(&cv, "attemptUpdate", "next", &[("next", "next")]),
            &cv,
            "/body/frames/0/parameters/0",
            malformed,
        ),
    ];
    for (case, body, locus, suffix, (code, cause)) in cases {
        let body = ordered(body);
        let value = relations(std::slice::from_ref(&body));
        expect(
            case,
            &value,
            code,
            Some(cause),
            &at_node(at(&value, &body), suffix),
            Some(locus),
        );
    }
}

/// IR reading (FR-346): a frame entry over an operation whose declared
/// parameter names are not all present and unique cannot match, and refuses
/// `invalid_model_binding`/`malformed-declaration` at the entry. The domain
/// document is admitted as it is.
///
/// Tracing: TC-225
/// ACs: FR-346-AC-7
#[trace("TC-225", "FR-346-AC-7")]
#[test]
fn tc_225_an_operation_without_distinct_parameter_names_cannot_be_bound() {
    let rebase = |parameters: &[(&str, &str)]| {
        body(
            vec![],
            vec![],
            vec![frame(
                &config_version(),
                "rebase",
                &["f"],
                "self",
                parameters,
            )],
        )
    };
    let undeclared = |edit: fn(&mut Value)| {
        let mut document = domain_document();
        edit(&mut document["types"][0]["operations"][1]["params"]);
        document
    };
    let cases: [(&str, Value, Value); 4] = [
        (
            "a parameter with no name",
            undeclared(|params| {
                params[1].as_object_mut().expect("parameter").remove("name");
            }),
            ordered(rebase(&[("from", "a"), ("to", "b")])),
        ),
        (
            "a parameter whose name is not a string",
            undeclared(|params| params[1]["name"] = json!(5)),
            ordered(rebase(&[("from", "a"), ("to", "b")])),
        ),
        (
            "two parameters of one name",
            undeclared(|params| params[1]["name"] = json!("from")),
            ordered(rebase(&[("from", "a"), ("to", "b")])),
        ),
        (
            "two parameters of one name bound twice",
            undeclared(|params| params[1]["name"] = json!("from")),
            ordered(rebase(&[("from", "a"), ("from", "b")])),
        ),
    ];
    for (case, document, relation) in cases {
        let value = with_nodes(base_over(&document), vec![relation_node(&relation)]);
        let refusal = match read_over(&value, &document) {
            CheckedPackageV2ReadResult::Refused(refusal) => refusal,
            other => panic!("{case}: expected a refusal, read {other:?}"),
        };
        assert_eq!(refusal.code, Code::InvalidModelBinding, "{case}");
        assert_eq!(refusal.cause, Some(Cause::MalformedDeclaration), "{case}");
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(at_node(at(&value, &relation), "/body/frames/0").as_str()),
            "{case}"
        );
        assert_eq!(
            refusal.locus.as_ref().map(|id| &*id.digest),
            Some(config_version().as_str()),
            "{case}"
        );
    }
}

// AC-8.

/// Tracing: TC-225
/// ACs: FR-346-AC-8
#[trace("TC-225", "FR-346-AC-8")]
#[test]
fn tc_225_a_second_binding_of_one_key_refuses_at_the_second_entry() {
    let cv = config_version();
    let conflict = Some(Cause::ConflictingBinding);
    // Two entries of one key in one node, for each of the three keys.
    let cases = [
        (
            "an object key",
            body(
                vec![object(&cv, &["a"], &[]), object(&cv, &["b"], &[])],
                vec![],
                vec![],
            ),
            "/body/objects/1",
            cv.as_str(),
        ),
        (
            "a population key",
            body(
                vec![],
                vec![
                    population(POPULATION_A, &["a"]),
                    population(POPULATION_A, &["b"]),
                ],
                vec![],
            ),
            "/body/populations/1",
            POPULATION_A,
        ),
        (
            "an operation key",
            body(
                vec![],
                vec![],
                vec![
                    frame(&cv, "attemptUpdate", &["f"], "self", &[("next", "n")]),
                    frame(&cv, "attemptUpdate", &["g"], "self", &[("next", "n")]),
                ],
            ),
            "/body/frames/1",
            cv.as_str(),
        ),
    ];
    for (case, body, suffix, locus) in cases {
        let value = relations(std::slice::from_ref(&body));
        expect(
            case,
            &value,
            Code::InvalidModelBinding,
            conflict,
            &at_node(at(&value, &body), suffix),
            Some(locus),
        );
    }
    // Two nodes: the second in ascending node-id order refuses, for each key.
    let pairs = [
        (
            "an object key across two nodes",
            body(vec![object(&cv, &["a"], &[])], vec![], vec![]),
            body(vec![object(&cv, &["b"], &[])], vec![], vec![]),
            "/body/objects/0",
            cv.as_str(),
        ),
        (
            "a population key across two nodes",
            body(vec![], vec![population(POPULATION_A, &["a"])], vec![]),
            body(vec![], vec![population(POPULATION_A, &["b"])], vec![]),
            "/body/populations/0",
            POPULATION_A,
        ),
        (
            "an operation key across two nodes",
            body(
                vec![],
                vec![],
                vec![frame(
                    &cv,
                    "attemptUpdate",
                    &["f"],
                    "self",
                    &[("next", "n")],
                )],
            ),
            body(
                vec![],
                vec![],
                vec![frame(
                    &cv,
                    "attemptUpdate",
                    &["g"],
                    "self",
                    &[("next", "n")],
                )],
            ),
            "/body/frames/0",
            cv.as_str(),
        ),
    ];
    for (case, first, second, suffix, locus) in pairs {
        let value = relations(&[first.clone(), second.clone()]);
        let mut by_key = [digest_of(&first), digest_of(&second)];
        by_key.sort();
        expect(
            case,
            &value,
            Code::InvalidModelBinding,
            conflict,
            &at_node(position(&value, &by_key[1]), suffix),
            Some(locus),
        );
    }
}

// AC-9.

/// A package holding a frame node, and an anchor over `context`.
fn with_state(value: Value, frame_context: &str, anchor_context: &str) -> Value {
    with_nodes(
        value,
        vec![
            state_frame(frame_context),
            operation_anchor(anchor_context, "attemptUpdate"),
        ],
    )
}

/// A `call` application node: every earlier step admits it, and the
/// operation step refuses it.
fn call_node() -> Value {
    let call = application("call", "quire.op.function.call");
    plain(
        &digest_of(&call),
        "expression",
        "call",
        INTEGER,
        &[],
        "generated",
        call,
    )
}

fn settled_value(mut value: Value) -> Value {
    settle(&mut value);
    value
}

/// The path a package is refused at.
fn refusal_path(case: &str, value: &Value) -> String {
    refused(case, value)
        .path
        .map(|path| path.as_str().to_owned())
        .expect("a pointer")
}

/// A relation body with a member defect: a `fields` name the type does not
/// expose.
fn member_defect(name: &str) -> Value {
    body(
        vec![object(&config_version(), &["t"], &[(name, "m")])],
        vec![],
        vec![],
    )
}

/// A relation body with a target defect: a population naming an object type.
fn target_defect() -> Value {
    body(vec![], vec![population(&config_version(), &["p"])], vec![])
}

/// Tracing: TC-225
/// ACs: FR-346-AC-9
#[trace("TC-225", "FR-346-AC-9")]
#[test]
fn tc_225_a_defect_of_an_earlier_step_is_reported_before_the_abstraction_step() {
    let defective = member_defect("missing");
    let defective_at = |value: &Value| at_node(at(value, &defective), "/body/objects/0/fields/0");
    // The abstraction defect alone refuses at the abstraction step.
    let alone = relations(std::slice::from_ref(&defective));
    assert_eq!(
        refusal_path("abstraction alone", &alone),
        defective_at(&alone)
    );
    // A frame defect: the frame's semantic_type is no object type.
    let frame_defect = with_nodes(alone.clone(), vec![state_frame(INTEGER)]);
    let frame_at = position(&frame_defect, FRAME);
    expect(
        "a frame defect and an abstraction defect",
        &frame_defect,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &at_node(frame_at, "/semantic_type"),
        None,
    );
    // A stale abstraction node_id and a frame defect: the frame defect.
    let stale = mutated(
        frame_defect.clone(),
        at(&frame_defect, &defective),
        |node| {
            node["node_id"] = node_id(&"9".repeat(64));
        },
    );
    assert_eq!(
        refusal_path("a stale node_id and a frame defect", &stale),
        at_node(frame_at, "/semantic_type")
    );
    // A state defect: the anchor's context is no object type.
    let state_defect = with_state(alone.clone(), &config_version(), INTEGER);
    let anchor_at = position(&state_defect, ANCHOR);
    expect(
        "a state defect and an abstraction defect",
        &state_defect,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &at_node(anchor_at, "/body/members/0/value/target"),
        None,
    );
    // A temporal placement defect: a formula node whose body is no formula.
    let formula = plain(
        FORMULA,
        "temporal",
        "formula",
        INTEGER,
        &[],
        "expression",
        empty(),
    );
    let temporal_defect = with_nodes(alone, vec![formula]);
    expect(
        "a temporal defect and an abstraction defect",
        &temporal_defect,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &at_node(position(&temporal_defect, FORMULA), ""),
        None,
    );
}

/// Tracing: TC-225
/// ACs: FR-346-AC-9
#[trace("TC-225", "FR-346-AC-9")]
#[test]
fn tc_225_an_abstraction_defect_is_reported_before_the_operation_step() {
    let call = call_node();
    // The operation defect alone: a refusal located at the call node.
    let alone = settled_value(with_nodes(base(), vec![call.clone()]));
    let call_at = nodes(&alone).len() - 1;
    let path = refusal_path("the operation defect alone", &alone);
    assert!(
        path.starts_with(&at_node(call_at, "")),
        "the operation step refuses the call node: {path}"
    );
    // With an abstraction defect the abstraction step refuses first.
    let defective = member_defect("missing");
    let both = settled_value(with_nodes(alone, vec![relation_node(&defective)]));
    assert_eq!(
        refusal_path("an abstraction and an operation defect", &both),
        at_node(at(&both, &defective), "/body/objects/0/fields/0")
    );
    // An ordinary-class application at an abstraction node's body root and an
    // operation defect: the body-root refusal.
    let (rooted, rooted_at) = rooted_in(application("call", "quire.op.function.call"));
    let both = settled_value(with_nodes(rooted, vec![call_node()]));
    expect(
        "a body root and an operation defect",
        &both,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &at_node(rooted_at, ""),
        None,
    );
}

/// Tracing: TC-225
/// ACs: FR-346-AC-9
#[trace("TC-225", "FR-346-AC-9")]
#[test]
fn tc_225_the_checks_of_one_node_run_in_identity_order_target_member_order() {
    let stale = |value: Value, at_body: &Value| {
        let n = at(&value, at_body);
        mutated(value, n, |node| node["node_id"] = node_id(&"9".repeat(64)))
    };
    // An identity defect and a target defect: the identity defect, located at
    // the node, not the target.
    let target = target_defect();
    let value = stale(relations(std::slice::from_ref(&target)), &target);
    let n = position(&value, &"9".repeat(64));
    expect(
        "an identity defect and a target defect",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &at_node(n, ""),
        None,
    );
    // A target defect and a member defect: the target defect, though the
    // member defect's entry comes first in body order.
    let both = body(
        vec![object(&config_version(), &["t"], &[("missing", "m")])],
        vec![population(&config_version(), &["p"])],
        vec![],
    );
    let value = relations(std::slice::from_ref(&both));
    expect(
        "a target defect and a member defect",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &at_node(at(&value, &both), "/body/populations/0"),
        Some(&config_version()),
    );
    // An order defect and a target defect: the order defect.
    let mut types = [config_version(), key(DRAFT)];
    types.sort();
    let reversed = body(
        vec![
            object(&types[1], &["t"], &[]),
            object(&types[0], &["t"], &[]),
        ],
        vec![population(&config_version(), &["p"])],
        vec![],
    );
    let value = relations(std::slice::from_ref(&reversed));
    expect(
        "an order defect and a target defect",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &at_node(at(&value, &reversed), "/body/objects"),
        None,
    );
}

/// Tracing: TC-225
/// ACs: FR-346-AC-9
#[trace("TC-225", "FR-346-AC-9")]
#[test]
fn tc_225_nodes_report_in_node_id_order_and_collisions_only_after_every_node() {
    // Two defective nodes: the lower node_id node's own defect.
    // The nodes are placed in the graph in descending node-key order, so the
    // lower key is the later position and a walk in position order differs.
    let (one, other) = (member_defect("missingOne"), member_defect("missingTwo"));
    let (lower, higher) = if digest_of(&one) < digest_of(&other) {
        (one, other)
    } else {
        (other, one)
    };
    let value = relations(&[higher.clone(), lower.clone()]);
    assert!(
        at(&value, &lower) > at(&value, &higher),
        "the lower key is the later position"
    );
    assert_eq!(
        refusal_path("two defective nodes", &value),
        at_node(at(&value, &lower), "/body/objects/0/fields/0")
    );
    // Two nodes that collide on an object key, and a third carrying a member
    // defect: the member defect, wherever the defective node sorts.
    let cv = config_version();
    let first = body(vec![object(&cv, &["a"], &[])], vec![], vec![]);
    let second = body(vec![object(&cv, &["b"], &[])], vec![], vec![]);
    let colliding = relations(&[first.clone(), second.clone()]);
    let mut by_key = [digest_of(&first), digest_of(&second)];
    by_key.sort();
    assert_eq!(
        refusal_path("the collision alone", &colliding),
        at_node(position(&colliding, &by_key[1]), "/body/objects/0"),
    );
    let defect = member_defect("missing");
    let value = relations(&[first, second, defect.clone()]);
    assert_eq!(
        refusal_path("a collision and a member defect", &value),
        at_node(at(&value, &defect), "/body/objects/0/fields/0"),
        "a key collision is reported after every node's own checks held"
    );
}

// AC-10.

/// Tracing: TC-225
/// ACs: FR-346-AC-10
#[trace("TC-225", "FR-346-AC-10")]
#[test]
fn tc_225_a_changed_binding_changes_the_node_key_and_the_package_id() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let package = admitted("the minimal body", &value);
    // The same body gives the same node_id on every read.
    let again = admitted("the same body again", &value);
    let n = at(&value, &minimal);
    assert_eq!(
        package.graph().nodes[n].node_id,
        again.graph().nodes[n].node_id
    );
    assert_eq!(package.package_id(), again.package_id());
    // One rust_field changed and the node_id recomputed.
    let mut changed = minimal.clone();
    changed["objects"][0]["fields"][0]["rust_field"] = json!("other");
    assert_ne!(digest_of(&changed), digest_of(&minimal));
    let rekeyed = rebodied(value.clone(), n, changed.clone());
    let other = admitted("one rust_field changed", &rekeyed);
    let other_at = at(&rekeyed, &changed);
    assert_eq!(
        other.graph().nodes[other_at].node_id,
        typed_node_id(&digest_of(&changed))
    );
    assert_ne!(
        other.graph().nodes[other_at].node_id,
        package.graph().nodes[n].node_id
    );
    assert_ne!(other.package_id(), package.package_id());
    // The same change without recomputing node_id refuses as in AC-4.
    let stale = mutated(value, n, |node| {
        node["body"]["objects"][0]["fields"][0]["rust_field"] = json!("other");
    });
    expect(
        "the change without a new node_id",
        &stale,
        Code::InvalidSemanticGraph,
        None,
        &at_node(n, ""),
        Some(&digest_of(&minimal)),
    );
}

// Lowering.

/// An admitted relation node reaches lowering as any admitted node does: the
/// lowering outcome for abstraction relations beyond that is an open question
/// of FR-346.
///
/// Tracing: TC-225
#[trace("TC-225")]
#[test]
fn tc_225_an_admitted_relation_node_lowers_like_any_admitted_node() {
    let minimal = minimal();
    let value = relations(std::slice::from_ref(&minimal));
    let package = admitted("the minimal body", &value);
    let n = at(&value, &minimal);
    let id = typed_node_id(&digest_of(&minimal));
    let profile = CompleteLoweringProfileV2 {
        supported_tags: [CheckedNodeTag::Correspondence, CheckedNodeTag::Model]
            .into_iter()
            .collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let result = package.lower(std::slice::from_ref(&id), &profile);
    let [CompleteLoweringRecordV2::Lowered { node }] = result.records.as_slice() else {
        panic!("expected one lowered record, read {:?}", result.records);
    };
    assert_eq!(node.node_tag, CheckedNodeTag::Correspondence);
    assert_eq!(node.node.node_id, id);
    assert_eq!(
        serde_json::to_value(&node.node).expect("node"),
        nodes(&value)[n]
    );
    assert_eq!(node.dependencies, vec![typed_node_id(&config_version())]);
}
