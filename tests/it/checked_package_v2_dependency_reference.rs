// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-289: the `dependency_reference` term (QSpec FR-322-AC-36 and
//! FR-322-AC-37, native-diagnostics step 7). A call to a function a
//! dependency package declares carries `{term: "dependency_reference",
//! package, node}` as its callee; the term enters the calling node's id, is
//! never listed in `dependencies`, and resolves against the dependency
//! package the caller supplies.
//!
//! The hand-built tests below run everywhere; both packages are built here,
//! node by node, from this crate's own fixture. `dependency_reference_vectors`
//! reads QSpec's `node-identity-vectors.json` at run time from the checkout
//! `QSPEC_DIR` names and nothing of QSpec is copied into this repository. It
//! skips (and passes) when `QSPEC_DIR` is unset; `make qspec-vectors`
//! requires it.

use crate::support::checked_package::{
    admitted_dependency, canonical, domain_package_digest, domain_package_document, family_key,
    node_id, nominal_package, pointer, read_with_dependencies, rebuild_source_map,
    refresh_identity, rekey, rekey_application_node, sha256_hex, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageRefusal, CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const LIBRARY: &str = "test/geometry";
const VERSION: &str = "1";
/// The `node` a well-formed callee names when no test says otherwise.
const OTHER_NODE: &str = "5353535353535353535353535353535353535353535353535353535353535353";

/// One dependency package: its document, the admitted package the reader
/// hands back and the digest of its `package_id`.
struct Dependency {
    package: CheckedPackageV2,
    digest: String,
}

/// A graph node with an empty aggregate body; `declared` is its qualified
/// name when it is a source declaration.
fn node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    declared: Option<&[&str]>,
) -> Value {
    let mut node = json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "aggregate", "members": []},
    });
    if let Some(name) = declared {
        node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
        node["declaration"] = json!({"qualified_name": name});
    }
    node
}

/// The dependency function's node: the fixture's `function` node, declared
/// as `twice`, over `Int[..]` (`bounded_domain`) and returning Boolean.
fn function_key() -> String {
    family_key("8080")
}

fn boolean_key() -> String {
    family_key("aaaa")
}

fn integer_range_key() -> String {
    family_key("cccc")
}

fn record_key() -> String {
    "7a".repeat(32)
}

/// A dependency package: the fixture's own graph with its `function` node
/// declared, taking a bounded integer and returning Boolean, after `edit`
/// changed the nodes.
fn dependency_document(edit: impl FnOnce(&mut Vec<Value>)) -> Value {
    let mut package = v2_all_families();
    let mut nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    let function = nodes
        .iter_mut()
        .find(|node| node["node_id"]["digest"] == function_key())
        .expect("the fixture's function node");
    *function = node(
        &function_key(),
        "function",
        "pure_function",
        &boolean_key(),
        &[&integer_range_key()],
        Some(&["twice"]),
    );
    edit(&mut nodes);
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    package
}

fn build_dependency(edit: impl FnOnce(&mut Vec<Value>)) -> Dependency {
    let (digest, package) = admitted_dependency(&dependency_document(edit));
    Dependency { package, digest }
}

fn dependency_term(package_digest: &str, node_digest: &str) -> Value {
    json!({
        "term": "dependency_reference",
        "package": {
            "domain": "quire.package.semantic/v2", "algorithm": "sha256", "digest": package_digest,
        },
        "node": node_id(node_digest),
    })
}

fn selection(digest: &str) -> Value {
    json!({
        "identity": LIBRARY,
        "version": VERSION,
        "package_id": {
            "domain": "quire.package.semantic/v2", "algorithm": "sha256", "digest": digest,
        },
    })
}

/// The position of the fixture's `function` call node.
fn call_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "function" && node["body"]["term"] == "application")
        .expect("the fixture's function call node")
}

/// The fixture package whose `function` call node calls `callee` (an
/// argument-0 term) with the selections `entries` locked, its application
/// key re-derived.
fn importing(entries: Vec<Value>, callee: Value) -> Value {
    importing_with(entries, |arguments| arguments[0] = callee)
}

fn importing_with(entries: Vec<Value>, edit: impl FnOnce(&mut Vec<Value>)) -> Value {
    let mut package = v2_all_families();
    package["lock"]["dependency_selections"] = Value::Array(entries);
    let position = call_position(&package);
    let call = &mut package["semantic_graph"]["nodes"][position];
    let mut arguments = call["body"]["arguments"]
        .as_array()
        .expect("arguments")
        .clone();
    edit(&mut arguments);
    call["body"]["arguments"] = Value::Array(arguments);
    // FR-322: the term is no `reference`, so it contributes no dependency.
    call["dependencies"] = json!([]);
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    package
}

fn read(package: &Value, dependency: &Dependency) -> CheckedPackageV2ReadResult {
    read_with_dependencies(package, &[(LIBRARY, VERSION, &dependency.package)])
}

fn refused(result: CheckedPackageV2ReadResult) -> CheckedPackageRefusal {
    match result {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn expect_refusal(
    refusal: &CheckedPackageRefusal,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    path: &str,
) {
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(refusal.path, Some(pointer(path)), "{refusal:?}");
}

fn call_path(package: &Value, tail: &str) -> String {
    format!(
        "/semantic_graph/nodes/{}/body{tail}",
        call_position(package)
    )
}

/// The importing package calling the dependency's `twice`.
fn calling(dependency: &Dependency, callee_node: &str) -> Value {
    importing(
        vec![selection(&dependency.digest)],
        dependency_term(&dependency.digest, callee_node),
    )
}

/// Tracing: TC-048
/// ACs: FR-038-AC-35
#[trace("TC-048", "FR-038-AC-35")]
#[test]
fn tc_048_a_dependency_callee_is_admitted_and_never_a_dependency() {
    let dependency = build_dependency(|_| {});
    let package = calling(&dependency, &function_key());
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package, &dependency) else {
        panic!("a call through a dependency_reference callee admits");
    };
    // The callee is in the node body, carried verbatim, and lists no
    // dependency.
    let position = call_position(&package);
    let call = &admitted.graph().nodes[position];
    assert_eq!(
        call.body["arguments"][0]["term"],
        json!("dependency_reference")
    );
    assert!(call.dependencies.is_empty());
    assert_eq!(
        serde_json::to_value(&call.body).expect("body")["arguments"][0],
        dependency_term(&dependency.digest, &function_key())
    );

    // Listing the target as a dependency of the calling node refuses.
    let mut listed = package.clone();
    listed["semantic_graph"]["nodes"][position]["dependencies"] = json!([node_id(&function_key())]);
    refresh_identity(&mut listed);
    let refusal = refused(read(&listed, &dependency));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{position}/dependencies"),
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-35
#[trace("TC-048", "FR-038-AC-35")]
#[test]
fn tc_048_the_term_enters_the_application_node_identity() {
    let dependency = build_dependency(|_| {});
    let package = calling(&dependency, &function_key());
    let position = call_position(&package);
    // Changing only the term's `node` or only its `package`, with the key
    // left as it was, is a stale key: both enter the preimage.
    let mut renoded = package.clone();
    renoded["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["node"]["digest"] =
        json!(OTHER_NODE);
    refresh_identity(&mut renoded);
    let refusal = refused(read(&renoded, &dependency));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::StaleNodeKey),
        &format!("/semantic_graph/nodes/{position}/node_id"),
    );
    let mut repackaged = package.clone();
    repackaged["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["package"]["digest"] =
        json!("5555555555555555555555555555555555555555555555555555555555555555");
    refresh_identity(&mut repackaged);
    let refusal = refused(read(&repackaged, &dependency));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::StaleNodeKey),
        &format!("/semantic_graph/nodes/{position}/node_id"),
    );
    // Rekeying each gives the node a different id, and distinct ones.
    let id = |package: &Value| package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let mut node_rekeyed = renoded.clone();
    rekey_application_node(&mut node_rekeyed, position);
    let mut package_rekeyed = repackaged.clone();
    rekey_application_node(&mut package_rekeyed, position);
    assert_ne!(id(&node_rekeyed), id(&package));
    assert_ne!(id(&package_rekeyed), id(&package));
    assert_ne!(id(&node_rekeyed), id(&package_rekeyed));
}

/// Tracing: TC-048
/// ACs: FR-038-AC-36
#[trace("TC-048", "FR-038-AC-36")]
#[test]
fn tc_048_the_term_is_admitted_only_as_a_function_call_callee() {
    let dependency = build_dependency(|_| {});
    let term = dependency_term(&dependency.digest, &function_key());
    // As a non-callee argument of the call.
    let extra = importing_with(vec![selection(&dependency.digest)], |arguments| {
        arguments[0] = term.clone();
        arguments.push(term.clone());
    });
    expect_refusal(
        &refused(read(&extra, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&extra, "/arguments/1"),
    );
    // As the only argument of a call of a local function: not a callee of a
    // dependency call... argument 0 of another operation.
    let mut other_operation = importing(vec![selection(&dependency.digest)], term.clone());
    let position = call_position(&other_operation);
    let temporal = other_operation["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "temporal" && node["body"]["term"] == "application")
        .expect("the fixture's temporal clause node");
    other_operation["semantic_graph"]["nodes"][temporal]["body"]["arguments"] = json!([term]);
    refresh_identity(&mut other_operation);
    assert_ne!(temporal, position);
    // The temporal clause's key covers its arguments; the stale key is the
    // earlier refusal, so re-derive it.
    rekey_application_node(&mut other_operation, temporal);
    refresh_identity(&mut other_operation);
    expect_refusal(
        &refused(read(&other_operation, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{temporal}/body/arguments/0"),
    );
    // As a node body root and inside an aggregate.
    let mut root = importing(vec![selection(&dependency.digest)], term.clone());
    let mut expression = node(
        &"9a".repeat(32),
        "expression",
        "reference",
        &boolean_key(),
        &[],
        None,
    );
    expression["body"] = term.clone();
    let count = root["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    root["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(expression);
    rebuild_source_map(&mut root);
    refresh_identity(&mut root);
    expect_refusal(
        &refused(read(&root, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{count}/body"),
    );
    let mut aggregate = root.clone();
    aggregate["semantic_graph"]["nodes"][count]["body"] =
        json!({"term": "aggregate", "members": [term]});
    refresh_identity(&mut aggregate);
    expect_refusal(
        &refused(read(&aggregate, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{count}/body/members/0"),
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-36
#[trace("TC-048", "FR-038-AC-36")]
#[test]
fn tc_048_a_malformed_term_refuses_at_the_member_at_fault() {
    let dependency = build_dependency(|_| {});
    let good = dependency_term(&dependency.digest, &function_key());
    let mut bare = good.clone();
    bare["package"] = json!(dependency.digest);
    let mut other_domain = good.clone();
    other_domain["package"]["domain"] = json!("quire.source.bytes/v1");
    let mut short_digest = good.clone();
    short_digest["package"]["digest"] = json!("abc");
    let mut no_node = good.clone();
    no_node.as_object_mut().expect("term").remove("node");
    let mut extra = good.clone();
    extra["target"] = good["node"].clone();
    let mut node_domain = good.clone();
    node_domain["node"]["domain"] = json!("quire.package.semantic/v2");
    for (name, term, at) in [
        ("bare package", bare, "/arguments/0/package"),
        ("other domain", other_domain, "/arguments/0/package"),
        ("short digest", short_digest, "/arguments/0/package"),
        ("missing node", no_node, "/arguments/0"),
        ("extra member", extra, "/arguments/0"),
        ("node in another domain", node_domain, "/arguments/0/node"),
    ] {
        let package = importing(vec![selection(&dependency.digest)], term);
        expect_refusal(
            &refused(read(&package, &dependency)),
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            None,
            &call_path(&package, at),
        );
        assert!(!name.is_empty());
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-37
#[trace("TC-048", "FR-038-AC-37")]
#[test]
fn tc_048_the_selected_dependency_is_supplied_and_binds_to_its_entry() {
    let dependency = build_dependency(|_| {});
    let package = calling(&dependency, &function_key());
    // No package supplied: refused at the entry, before any term is read.
    let refusal = refused(read_with_dependencies(&package, &[]));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::MissingImport,
        Some(CheckedPackageRefusalCause::MissingSelection),
        "/lock/dependency_selections/0",
    );
    // Supplied under another identity is no package for this entry.
    let refusal = refused(read_with_dependencies(
        &package,
        &[("test/other", VERSION, &dependency.package)],
    ));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::MissingImport,
        Some(CheckedPackageRefusalCause::MissingSelection),
        "/lock/dependency_selections/0",
    );
    // A version that differs from the entry's.
    let refusal = refused(read_with_dependencies(
        &package,
        &[(LIBRARY, "2", &dependency.package)],
    ));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::StaleDependency,
        Some(CheckedPackageRefusalCause::RevisionMismatch),
        "/lock/dependency_selections/0/version",
    );
    // A package whose own package_id is not the entry's.
    let other = crate::support::checked_package::positive_operation_identities();
    let (_, other) = admitted_dependency(&other);
    let refusal = refused(read_with_dependencies(
        &package,
        &[(LIBRARY, VERSION, &other)],
    ));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::StaleDependency,
        Some(CheckedPackageRefusalCause::ByteDigestMismatch),
        "/lock/dependency_selections/0/package_id/digest",
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-37
#[trace("TC-048", "FR-038-AC-37")]
#[test]
fn tc_048_a_reference_resolves_in_the_order_selection_name_operator() {
    let dependency = build_dependency(|_| {});
    let entries = || vec![selection(&dependency.digest)];
    let term_at =
        |package: &Value, member: &str| call_path(package, &format!("/arguments/0{member}"));

    // A `package` no selection names.
    let package = importing(entries(), dependency_term(&"6".repeat(64), &function_key()));
    let refusal = refused(read(&package, &dependency));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::MissingDeclaration,
        Some(CheckedPackageRefusalCause::MissingSelection),
        &term_at(&package, "/package"),
    );
    assert!(refusal.locus.is_some(), "located at the calling node");

    // A `node` naming no node of the dependency, and one that carries no
    // `declaration`.
    for node in [OTHER_NODE.to_owned(), integer_range_key()] {
        let package = calling(&dependency, &node);
        expect_refusal(
            &refused(read(&package, &dependency)),
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingName),
            &term_at(&package, "/node"),
        );
    }

    // A declared node that is no `function`.
    let with_record = build_dependency(|nodes| {
        nodes.push(node(
            &record_key(),
            "composite_type",
            "record",
            &boolean_key(),
            &[&boolean_key()],
            Some(&["R"]),
        ));
    });
    let package = calling(&with_record, &record_key());
    expect_refusal(
        &refused(read(&package, &with_record)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&package, "/arguments/0"),
    );
}

/// The function `twice`'s signature after `edit` added type nodes to the
/// graph: `signature` is what its `dependencies` list, `result` its
/// `semantic_type`.
fn signature(signature: &[&str], result: &str, edit: impl FnOnce(&mut Vec<Value>)) -> Dependency {
    build_dependency(|nodes| {
        nodes.push(node(
            &record_key(),
            "composite_type",
            "record",
            &boolean_key(),
            &[&boolean_key()],
            Some(&["R"]),
        ));
        edit(nodes);
        let function = nodes
            .iter_mut()
            .find(|node| node["node_id"]["digest"] == function_key())
            .expect("the function");
        function["semantic_type"] = node_id(result);
        function["dependencies"] = Value::Array(signature.iter().map(|key| node_id(key)).collect());
    })
}

/// Tracing: TC-048
/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_a_signature_type_that_is_declared_or_model_owned_refuses() {
    let set_key = "7b".repeat(32);
    let tuple_key = "7c".repeat(32);
    let reference_key = "7d".repeat(32);
    let model = family_key("1515");
    let cases: Vec<(&str, Dependency)> = vec![
        (
            "a parameter that is a declared record",
            signature(&[&record_key()], &boolean_key(), |_| {}),
        ),
        (
            "a result that is a declared record",
            signature(&[&integer_range_key()], &record_key(), |_| {}),
        ),
        (
            "a set of a declared record",
            signature(&[&set_key], &boolean_key(), |nodes| {
                nodes.push(node(
                    &set_key,
                    "composite_type",
                    "set",
                    &boolean_key(),
                    &[&record_key()],
                    None,
                ));
            }),
        ),
        (
            "a tuple holding a declared record",
            signature(&[&tuple_key], &boolean_key(), |nodes| {
                nodes.push(node(
                    &tuple_key,
                    "composite_type",
                    "tuple",
                    &boolean_key(),
                    &[&integer_range_key(), &record_key()],
                    None,
                ));
            }),
        ),
        (
            "a reference to a model type",
            signature(&[&reference_key], &boolean_key(), |nodes| {
                nodes.push(node(
                    &reference_key,
                    "composite_type",
                    "reference",
                    &boolean_key(),
                    &[&model],
                    None,
                ));
            }),
        ),
    ];
    for (name, dependency) in cases {
        let package = calling(&dependency, &function_key());
        let refusal = refused(read(&package, &dependency));
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped, "{name}");
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            "{name}"
        );
        assert_eq!(
            refusal.path,
            Some(pointer(&call_path(&package, "/arguments/0"))),
            "{name}"
        );
    }
    // The same shapes over package-independent types admit: a set of the
    // bounded integer.
    let ok = signature(&[&set_key], &boolean_key(), |nodes| {
        nodes.push(node(
            &set_key,
            "composite_type",
            "set",
            &boolean_key(),
            &[&integer_range_key()],
            None,
        ));
    });
    let package = calling(&ok, &function_key());
    assert!(
        matches!(read(&package, &ok), CheckedPackageV2ReadResult::Admitted(_)),
        "a set of a package-independent type admits"
    );
}

/// `node` with `body` as its body.
fn with_body(mut node: Value, body: Value) -> Value {
    node["body"] = body;
    node
}

fn reference_to(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

/// A `value`/`parameter` node typed at `semantic_type`, with the closed
/// parameter body (a text name and an integer level, each typed at a scalar
/// node the caller adds).
fn parameter(key: &str, semantic_type: &str, text: &str, integer: &str) -> Value {
    let literal = |type_key: &str, kind: &str, value: &str| json!({"term": "literal", "type": node_id(type_key), "value_kind": kind, "value": value});
    with_body(
        node(key, "value", "parameter", semantic_type, &[], None),
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": literal(text, "text", "x")},
            {"term": "binding", "name": "level", "value": literal(integer, "integer", "0")},
        ]}),
    )
}

/// The dependency function with `parameter` as a parameter node, its type
/// the declared record, listed as the caller says.
fn parameter_dependency(listed: bool, referenced: bool) -> Dependency {
    let (text, integer, parameter_key) = ("7e".repeat(32), "7f".repeat(32), "7d".repeat(32));
    let signed: &[&str] = if listed { &[&parameter_key] } else { &[] };
    signature(signed, &boolean_key(), |nodes| {
        nodes.push(node(&text, "scalar_type", "text", &text, &[], None));
        nodes.push(node(
            &integer,
            "scalar_type",
            "integer",
            &integer,
            &[],
            None,
        ));
        nodes.push(parameter(&parameter_key, &record_key(), &text, &integer));
        let function = nodes
            .iter_mut()
            .find(|node| node["node_id"]["digest"] == function_key())
            .expect("the function");
        if referenced {
            function["body"] = json!({
                "term": "aggregate", "members": [reference_to(&parameter_key)],
            });
        }
    })
}

/// Tracing: TC-048
/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_a_parameter_of_a_declared_type_refuses_however_the_function_reaches_it() {
    // Listed in `dependencies` only, referenced from the body only (a
    // non-application function body need not list what it references), and
    // both.
    for (name, listed, referenced) in [
        ("listed", true, false),
        ("body reference", false, true),
        ("both", true, true),
    ] {
        let dependency = parameter_dependency(listed, referenced);
        let package = calling(&dependency, &function_key());
        expect_refusal(
            &refused(read(&package, &dependency)),
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            &call_path(&package, "/arguments/0"),
        );
        assert!(!name.is_empty());
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_the_signature_closure_follows_body_references_and_semantic_types() {
    let (by_body, by_type) = ("7b".repeat(32), "7c".repeat(32));
    // A set no `dependencies` entry ties to the record: its body references
    // it.
    let body = signature(&[&by_body], &boolean_key(), |nodes| {
        nodes.push(with_body(
            node(&by_body, "composite_type", "set", &boolean_key(), &[], None),
            json!({"term": "aggregate", "members": [reference_to(&record_key())]}),
        ));
    });
    // A set whose semantic type is the record.
    let typed = signature(&[&by_type], &boolean_key(), |nodes| {
        nodes.push(node(
            &by_type,
            "composite_type",
            "set",
            &record_key(),
            &[],
            None,
        ));
    });
    for (name, dependency) in [("body reference", body), ("semantic type", typed)] {
        let package = calling(&dependency, &function_key());
        let refusal = refused(read(&package, &dependency));
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped, "{name}");
        assert_eq!(
            refusal.path,
            Some(pointer(&call_path(&package, "/arguments/0"))),
            "{name}"
        );
    }
}

/// A dependency package built on the recorded nominal package: its
/// dimension, unit and enum declarations, with a declared `twice` returning
/// `result` after `edit` changed the nodes.
fn nominal_dependency(
    mut package: Value,
    result: &str,
    edit: impl FnOnce(&mut Vec<Value>),
) -> Dependency {
    let mut nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    nodes.push(node(
        &function_key(),
        "function",
        "pure_function",
        result,
        &[],
        Some(&["twice"]),
    ));
    edit(&mut nodes);
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let (digest, admitted) = admitted_dependency(&package);
    Dependency {
        package: admitted,
        digest,
    }
}

fn node_key(package: &Value, position: usize) -> String {
    package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("key")
        .to_owned()
}

/// Tracing: TC-048
/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_a_quantity_over_a_declared_unit_refuses() {
    let recorded = v2_nominal();
    // The recorded package's unit is `Example::Metre`; the function returns
    // it.
    let unit = node_key(&recorded, 2);
    let dependency = nominal_dependency(recorded, &unit, |_| {});
    let package = calling(&dependency, &function_key());
    expect_refusal(
        &refused(read(&package, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&package, "/arguments/0"),
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_a_type_owned_by_a_domain_package_refuses_without_a_declaration() {
    // The recorded nominal package with its enum owned by a domain package
    // and carrying no `declaration`: only its owner makes it
    // package-dependent.
    let owner =
        json!({"kind": "model", "identity": "test/orders", "node": "ix://test/orders/Status"});
    let recorded = v2_nominal();
    let nodes = recorded["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let order = [1, 0, 3, 2];
    let mut preimages = order
        .iter()
        .map(|position| nodes[*position]["nominal_identity_preimage"].clone())
        .collect::<Vec<_>>();
    let keys = order
        .iter()
        .map(|position| node_key(&recorded, *position))
        .collect::<Vec<_>>();
    preimages[0]["owner"] = owner;
    let fresh = rekey(&mut preimages, &keys);
    let members = preimages.into_iter().zip(fresh).collect::<Vec<_>>();
    let mut owned = nominal_package(&members);
    let document = domain_package_document("test/orders", "1", Vec::new());
    owned["lock"]["model_selections"] = json!([{
        "identity": "test/orders", "version": "1",
        "digest_domain": "sha256-jcs", "digest": domain_package_digest(&document),
    }]);
    refresh_identity(&mut owned);
    let enum_key = node_key(&owned, 0);
    let dependency = nominal_dependency(owned, &enum_key, |nodes| {
        let declaration = &mut nodes[0];
        declaration["occurrences"] = json!([{"role": "generated", "ordinal": 0}]);
        declaration
            .as_object_mut()
            .expect("node")
            .remove("declaration");
    });
    let package = calling(&dependency, &function_key());
    expect_refusal(
        &refused(read(&package, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&package, "/arguments/0"),
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-36
#[trace("TC-048", "FR-038-AC-36")]
#[test]
fn tc_048_a_binding_value_refuses_and_a_nested_call_callee_admits() {
    let dependency = build_dependency(|_| {});
    let term = dependency_term(&dependency.digest, &function_key());
    let entries = || vec![selection(&dependency.digest)];
    // A binding's value is no callee.
    let bound = importing_with(entries(), |arguments| {
        arguments[0] = term.clone();
        arguments.push(json!({"term": "binding", "name": "x", "value": term.clone()}));
    });
    expect_refusal(
        &refused(read(&bound, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&bound, "/arguments/1/value"),
    );
    // A nested `function.call` takes the callee at its own argument 0.
    let nested_call = |arguments: Vec<Value>| {
        json!({
            "term": "application", "operator": "call",
            "operation": {
                "identity": "quire.op.function.call", "laws": [], "mode": null,
                "member": null, "leaves": [],
            },
            "result_type": node_id(&boolean_key()),
            "arguments": arguments,
        })
    };
    let nested = importing_with(entries(), |arguments| {
        arguments[0] = term.clone();
        arguments.push(nested_call(vec![term.clone()]));
    });
    assert!(
        matches!(
            read(&nested, &dependency),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "a nested function.call callee admits"
    );
    // The same term as the nested call's second argument is no callee.
    let misplaced = importing_with(entries(), |arguments| {
        arguments[0] = term.clone();
        arguments.push(nested_call(vec![term.clone(), term.clone()]));
    });
    expect_refusal(
        &refused(read(&misplaced, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&misplaced, "/arguments/1/arguments/1"),
    );
}

const VECTORS: &str = "proposals/checked-package-v2/node-identity-vectors.json";

/// A file of the QSpec checkout `QSPEC_DIR` names, or `None` when unset.
fn qspec(relative: &str) -> Option<Value> {
    let root = std::env::var_os("QSPEC_DIR")?;
    let path = std::path::Path::new(&root).join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    Some(serde_json::from_str(&text).expect("QSpec JSON"))
}

/// Tracing: TC-048
/// ACs: FR-038-AC-35, FR-038-AC-36
#[trace("TC-048", "FR-038-AC-35", "FR-038-AC-36")]
#[test]
fn dependency_reference_vectors() {
    let Some(vectors) = qspec(VECTORS) else {
        println!("skipped: QSPEC_DIR not set");
        return;
    };
    let call = vectors["operation_vectors"]
        .as_array()
        .expect("operation vectors")
        .iter()
        .find(|vector| vector["name"] == "function-call")
        .expect("QSpec publishes the function-call vector");
    let dependency = build_dependency(|_| {});
    let term = dependency_term(&dependency.digest, &function_key());

    // QSpec's application-node preimage for the call, with its type node the
    // fixture's Boolean and its callee the dependency reference. The id is
    // computed here from QSpec's published shape, not by the reader.
    let mut preimage = call["preimage"].clone();
    let published_type = preimage["semantic_type"].clone();
    let boolean = node_id(&boolean_key());
    preimage["semantic_type"] = boolean.clone();
    preimage["body"]["result_type"] = boolean;
    assert_eq!(
        published_type, call["preimage"]["body"]["result_type"],
        "the published call's type and result type are one node"
    );
    preimage["body"]["arguments"][0] = term.clone();
    let published_id = call["sha256"].as_str().expect("vector digest");
    assert_ne!(
        sha256_hex(&canonical(&preimage)),
        published_id,
        "a dependency callee keys apart"
    );

    // The fixture's call node is a `function`/`pure_function`; QSpec's vector
    // is `expression`/`call`. Derive the id on the fixture node's own tag and
    // form. The reader derives that same key: the package's call node, keyed
    // as QSpec's preimage keys it, admits with no stale-key refusal.
    preimage["node_tag"] = json!("function");
    preimage["semantic_form"] = json!("pure_function");
    let expected = sha256_hex(&canonical(&preimage));
    let package = importing(vec![selection(&dependency.digest)], term.clone());
    let position = call_position(&package);
    assert_eq!(
        package["semantic_graph"]["nodes"][position]["node_id"]["digest"],
        json!(expected),
        "the fixture keys the call as QSpec's preimage does"
    );
    assert!(
        matches!(
            read(&package, &dependency),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "the reader keys the dependency call as QSpec's preimage does"
    );

    // Changing only the `package` digest, then only the `node`, gives new,
    // distinct ids.
    let mut repackaged = preimage.clone();
    repackaged["body"]["arguments"][0]["package"]["digest"] = json!("55".repeat(32));
    let mut renoded = preimage.clone();
    renoded["body"]["arguments"][0]["node"]["digest"] = json!("54".repeat(32));
    let (repackaged, renoded) = (
        sha256_hex(&canonical(&repackaged)),
        sha256_hex(&canonical(&renoded)),
    );
    assert_ne!(repackaged, expected);
    assert_ne!(renoded, expected);
    assert_ne!(renoded, repackaged);

    // The four mutations QSpec's schemas refuse, each refused by the reader
    // at the member at fault.
    let mut bare = term.clone();
    bare["package"] = json!(dependency.digest);
    let mut other_domain = term.clone();
    other_domain["package"]["domain"] = json!("quire.source.bytes/v1");
    let mut no_node = term.clone();
    no_node.as_object_mut().expect("term").remove("node");
    let mut extra = term.clone();
    extra["target"] = term["node"].clone();
    for (name, mutation, at) in [
        ("bare-digest package", bare, "/arguments/0/package"),
        (
            "another digest domain",
            other_domain,
            "/arguments/0/package",
        ),
        ("missing node", no_node, "/arguments/0"),
        ("extra member", extra, "/arguments/0"),
    ] {
        let package = importing(vec![selection(&dependency.digest)], mutation);
        let refusal = refused(read(&package, &dependency));
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "{name}: {refusal:?}"
        );
        assert_eq!(
            refusal.path,
            Some(pointer(&call_path(&package, at))),
            "{name}: {refusal:?}"
        );
    }
    println!("conformance: dependency-reference-vectors function-call key + 4 mutations");
}
