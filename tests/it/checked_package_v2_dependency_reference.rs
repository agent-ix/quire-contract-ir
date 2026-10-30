// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-289: the `dependency_reference` term (QSpec FR-322-AC-36 and
//! FR-322-AC-37, native-diagnostics step 7). A call to a function a
//! dependency package declares carries `{term: "dependency_reference",
//! package, node}` as its callee; the term enters the calling node's id, is
//! never listed in `dependencies`, and resolves against the dependency
//! package the caller supplies.
//!
//! Both packages are built here, node by node, from this crate's own
//! fixture.

use crate::support::checked_package::{
    admitted_dependency, canonical, family_key, mint_ungrouped_structural_keys, node_id,
    nominal_package, over_body, pointer, read_with_dependencies, rebuild_source_map,
    refresh_identity, rekey, rekey_application_node, rename_node, select_model, settle, sha256_hex,
    source_owner, structural_key, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeTag, CheckedOccurrenceRole,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
    CompleteLoweringProfileV2, ContractDependencies, ContractDependencyInput,
    ContractOccurrenceKeyV1, ContractPackageReadResult, ContractPackageWireV1,
    ContractResultContextV1, ContractResultViewV1, ContractRootViewV1, LoweringProfileRef,
};
use serde_json::{json, Value};

const LIBRARY: &str = "test/geometry";
/// The `node` a well-formed callee names when no test says otherwise.
const OTHER_NODE: &str = "5353535353535353535353535353535353535353535353535353535353535353";

/// One dependency package: its document, the admitted package the reader
/// hands back and the digest of its `package_id`.
struct Dependency {
    package: CheckedPackageV2,
    digest: String,
    function_key: String,
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
        let source = crate::support::checked_package::fixture_source();
        node["owner"] = json!({
            "kind": "source", "authority": source["authority"],
            "identity": source["identity"],
        });
    }
    node
}

/// The dependency function's node: the fixture's `function` node, declared
/// as `twice`, over `Int[..]` (`bounded_domain`) and returning Boolean.
fn function_key() -> String {
    declared_key("function", "pure_function", &boolean_key(), "twice")
}

fn boolean_key() -> String {
    family_key("aaaa")
}

fn integer_range_key() -> String {
    family_key("cccc")
}

fn record_key() -> String {
    declared_key("composite_type", "record", &boolean_key(), "R")
}

fn declared_key(tag: &str, form: &str, semantic_type: &str, name: &str) -> String {
    sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": tag,
        "semantic_form": form, "semantic_type": node_id(semantic_type),
        "declaration": {"qualified_name": [name]}, "owner": source_owner(&crate::support::checked_package::fixture_source()),
        "recursion": null, "body": {"term": "aggregate", "members": []},
    })))
}

/// The anonymous `composite_type` node of `form` over `element`, self-typed,
/// its body the one `reference` and its key the derived one (FR-038-AC-134).
fn collection(form: &str, element: &str, dependencies: &[&str]) -> (String, Value) {
    let body = over_body(element);
    let key = structural_key("composite_type", form, None, &body);
    let collection = with_body(
        node(&key, "composite_type", form, &key, dependencies, None),
        body,
    );
    (key, collection)
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
        .find(|node| node["node_id"]["digest"] == family_key("8080"))
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
    rename_node(&mut package, &family_key("8080"), &function_key());
    mint_ungrouped_structural_keys(&mut package);
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    package
}

fn build_dependency(edit: impl FnOnce(&mut Vec<Value>)) -> Dependency {
    let document = dependency_document(edit);
    let function_key = declared_function_key(&document);
    let (digest, package) = admitted_dependency(&document);
    Dependency {
        package,
        digest,
        function_key,
    }
}

fn declared_function_key(package: &Value) -> String {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["declaration"]["qualified_name"] == json!(["twice"]))
        .expect("twice declaration")["node_id"]["digest"]
        .as_str()
        .expect("function digest")
        .to_owned()
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
    // Implements: FR-322. The term is no `reference`, so it contributes no dependency.
    call["dependencies"] = json!([]);
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    package
}

fn read(package: &Value, dependency: &Dependency) -> CheckedPackageV2ReadResult {
    read_with_dependencies(package, &[(LIBRARY, &dependency.package)])
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
    assert_eq!(
        refusal.loci[0].path.clone(),
        Some(pointer(path)),
        "{refusal:?}"
    );
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

#[trace("TC-047", "FR-035-AC-7", "TC-447", "FR-049-AC-5")]
#[test]
fn reached_view_retains_selected_dependency_pair_at_package_boundary() {
    let dependency = build_dependency(|_| {});
    let mut package = calling(&dependency, &dependency.function_key);
    let local_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|entry| entry["node_id"]["digest"] == family_key("8080"))
        .expect("local function");
    package["semantic_graph"]["nodes"][local_position] = node(
        &dependency.function_key,
        "function",
        "pure_function",
        &boolean_key(),
        &[],
        Some(&["twice"]),
    );
    rename_node(&mut package, &family_key("8080"), &dependency.function_key);
    let call = call_position(&package);
    package["semantic_graph"]["nodes"][call]["occurrences"][0]["role"] = json!("expression");
    rekey_application_node(&mut package, call);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let admitted = match read(&package, &dependency) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        other => panic!("selected source package must admit: {other:?}"),
    };
    let root: CheckedNodeId = serde_json::from_value(
        package["semantic_graph"]["nodes"][call_position(&package)]["node_id"].clone(),
    )
    .expect("call key");
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let result = admitted.lower(std::slice::from_ref(&root), &profile);
    let emitted = result.package.wire().expect("lowered package");
    let ContractRootViewV1::Lowered(view) = emitted.reached_closure(&root).expect("view") else {
        panic!("lowered call must have a view");
    };
    assert_eq!(view.selected_references.len(), 1);
    assert_eq!(
        view.selected_references[0].package,
        *dependency.package.package_id()
    );
    assert_eq!(
        view.selected_references[0].node,
        serde_json::from_value(node_id(&dependency.function_key)).expect("target key")
    );
    assert!(admitted
        .graph()
        .nodes
        .iter()
        .any(|entry| entry.node_id == view.selected_references[0].node));
    assert!(!view
        .nodes
        .iter()
        .any(|entry| entry.node.node_id == view.selected_references[0].node));
    let mut selected = ContractDependencies::new();
    selected.insert(
        LIBRARY,
        ContractDependencyInput {
            identity: LIBRARY,
            package: &dependency.package,
        },
    );
    let ContractPackageReadResult::Admitted(read_back) = ContractPackageWireV1::read(
        result.package.canonical_bytes().expect("bytes"),
        CheckedPackageReadLimits::bounded(),
        &selected,
    ) else {
        panic!("emitted package must read with selected dependency");
    };
    assert_eq!(
        read_back.reached_closure(&root).expect("view"),
        ContractRootViewV1::Lowered(view.clone())
    );
    let application = ContractOccurrenceKeyV1 {
        node_id: root.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let ContractResultViewV1::Lowered(contextual_view) = read_back
        .result_context(&root, &application, None, BoundedDomainForm::IntegerRange)
        .expect("admitted result context")
    else {
        panic!("selected external call remains lowered");
    };
    assert_eq!(contextual_view.context, ContractResultContextV1::Unnarrowed);
    assert_eq!(contextual_view.application, application);
    assert_eq!(
        contextual_view.reached.selected_references,
        view.selected_references
    );
    assert!(matches!(
        ContractPackageWireV1::read(
            result.package.canonical_bytes().expect("bytes"),
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
        ),
        ContractPackageReadResult::Refused(refusal) if refusal.code == CheckedPackageRefusalCode::MissingImport
    ));
    let mut wrong_identity = ContractDependencies::new();
    wrong_identity.insert(
        LIBRARY,
        ContractDependencyInput {
            identity: "test/other",
            package: &dependency.package,
        },
    );
    assert!(matches!(
        ContractPackageWireV1::read(
            result.package.canonical_bytes().expect("bytes"),
            CheckedPackageReadLimits::bounded(),
            &wrong_identity,
        ),
        ContractPackageReadResult::Refused(refusal) if refusal.code == CheckedPackageRefusalCode::MissingImport
    ));
    let (_, other) =
        admitted_dependency(&crate::support::checked_package::positive_operation_identities());
    let mut stale = ContractDependencies::new();
    stale.insert(
        LIBRARY,
        ContractDependencyInput {
            identity: LIBRARY,
            package: &other,
        },
    );
    assert!(matches!(
        ContractPackageWireV1::read(
            result.package.canonical_bytes().expect("bytes"),
            CheckedPackageReadLimits::bounded(),
            &stale,
        ),
        ContractPackageReadResult::Refused(refusal) if refusal.code == CheckedPackageRefusalCode::StaleDependency
    ));
}

/// ACs: FR-038-AC-35
#[trace("TC-048", "FR-038-AC-35", "FR-349-AC-1")]
#[test]
fn tc_048_a_dependency_callee_is_admitted_and_never_a_dependency() {
    let dependency = build_dependency(|_| {});
    let package = calling(&dependency, &dependency.function_key);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package, &dependency) else {
        panic!("a call through a dependency_reference callee admits");
    };
    // The callee is in the node body, carried verbatim, and lists no
    // dependency.
    let position = call_position(&package);
    let call = &admitted.graph().nodes[position];
    assert_eq!(
        serde_json::to_value(&call.body).expect("typed body")["arguments"][0]["term"],
        json!("dependency_reference")
    );
    assert!(call.dependencies.is_empty());
    assert_eq!(
        serde_json::to_value(&call.body).expect("body")["arguments"][0],
        dependency_term(&dependency.digest, &function_key())
    );
    let CheckedTerm::Application { arguments, .. } = admitted
        .term(&call.node_id)
        .expect("admitted call has a typed body")
    else {
        panic!("admitted call must be a typed application");
    };
    assert!(matches!(
        arguments.first(),
        Some(CheckedTerm::DependencyReference { package, node })
            if package.digest.as_ref() == dependency.digest.as_str()
                && node.digest.as_ref() == function_key().as_str()
    ));

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

#[trace("TC-048", "FR-038-AC-37")]
#[test]
fn transitive_only_dependency_reference_refuses_missing_selection() {
    let leaf = build_dependency(|_| {});
    let mut direct_wire = dependency_document(|_| {});
    direct_wire["lock"]["dependency_selections"] = json!([selection(&leaf.digest)]);
    refresh_identity(&mut direct_wire);
    let CheckedPackageV2ReadResult::Admitted(direct) =
        read_with_dependencies(&direct_wire, &[(LIBRARY, &leaf.package)])
    else {
        panic!("direct library with a leaf dependency admits");
    };
    let root = importing(
        vec![json!({"identity": "test/direct", "package_id": direct.package_id()})],
        dependency_term(&leaf.digest, &leaf.function_key),
    );
    let refusal = refused(read_with_dependencies(&root, &[("test/direct", &direct)]));
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
}

/// ACs: FR-038-AC-35
#[trace("TC-048", "FR-038-AC-35")]
#[test]
fn tc_048_the_term_enters_the_application_node_identity() {
    let dependency = build_dependency(|_| {});
    let package = calling(&dependency, &dependency.function_key);
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
    // The term is no `reference`, so the clause now has no dependency.
    other_operation["semantic_graph"]["nodes"][temporal]["dependencies"] = json!([]);
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
    let expression_key = structural_key("expression", "reference", Some(&boolean_key()), &term);
    let mut expression = node(
        &expression_key,
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
    let aggregate_key = structural_key(
        "expression",
        "reference",
        Some(&boolean_key()),
        &aggregate["semantic_graph"]["nodes"][count]["body"],
    );
    rename_node(&mut aggregate, &expression_key, &aggregate_key);
    refresh_identity(&mut aggregate);
    expect_refusal(
        &refused(read(&aggregate, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{count}/body/members/0"),
    );
}

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

/// ACs: FR-038-AC-37
#[trace("TC-048", "FR-038-AC-37")]
#[test]
fn tc_048_the_selected_dependency_is_supplied_and_binds_to_its_entry() {
    let dependency = build_dependency(|_| {});
    let package = calling(&dependency, &dependency.function_key);
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
        &[("test/other", &dependency.package)],
    ));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::MissingImport,
        Some(CheckedPackageRefusalCause::MissingSelection),
        "/lock/dependency_selections/0",
    );
    // A package whose own package_id is not the entry's.
    let other = crate::support::checked_package::positive_operation_identities();
    let (_, other) = admitted_dependency(&other);
    let refusal = refused(read_with_dependencies(&package, &[(LIBRARY, &other)]));
    expect_refusal(
        &refusal,
        CheckedPackageRefusalCode::StaleDependency,
        Some(CheckedPackageRefusalCause::ContentMismatch),
        "/lock/dependency_selections/0/package_id/digest",
    );
}

#[trace("TC-048", "FR-038-AC-37")]
#[test]
fn dependency_selection_checks_presence_then_binding_in_lock_order() {
    let dependency = build_dependency(|_| {});
    let mut second = selection(&"f".repeat(64));
    second["identity"] = json!("test/other");
    let package = importing(
        vec![selection(&dependency.digest), second],
        dependency_term(&dependency.digest, &dependency.function_key),
    );
    for (supplied, code, cause, path) in [
        (
            None,
            CheckedPackageRefusalCode::MissingImport,
            CheckedPackageRefusalCause::MissingSelection,
            "/lock/dependency_selections/0",
        ),
        (
            Some(&dependency.package),
            CheckedPackageRefusalCode::MissingImport,
            CheckedPackageRefusalCause::MissingSelection,
            "/lock/dependency_selections/1",
        ),
    ] {
        let dependencies = supplied.map_or_else(Vec::new, |dep| vec![(LIBRARY, dep)]);
        expect_refusal(
            &refused(read_with_dependencies(&package, &dependencies)),
            code,
            Some(cause),
            path,
        );
    }
    let (_, stale) =
        admitted_dependency(&crate::support::checked_package::positive_operation_identities());
    expect_refusal(
        &refused(read_with_dependencies(&package, &[(LIBRARY, &stale)])),
        CheckedPackageRefusalCode::StaleDependency,
        Some(CheckedPackageRefusalCause::ContentMismatch),
        "/lock/dependency_selections/0/package_id/digest",
    );
}

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

/// The function `twice` with parameter types in its body binding and a result
/// in its `semantic_type`.
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
        let parameter_keys: Vec<_> = signature
            .iter()
            .map(|type_key| {
                let parameter = parameter(type_key, &family_key("a1a1"), &family_key("a3a3"));
                let key = parameter["node_id"]["digest"]
                    .as_str()
                    .expect("parameter key")
                    .to_owned();
                nodes.push(parameter);
                key
            })
            .collect();
        if !parameter_keys.is_empty() {
            let call = nodes
                .iter_mut()
                .find(|node| node["body"]["term"] == "application")
                .expect("the fixture call");
            let arguments = call["body"]["arguments"].as_array_mut().expect("arguments");
            arguments.extend(parameter_keys.iter().map(|key| reference_to(key)));
            let function = nodes
                .iter_mut()
                .find(|node| node["node_id"]["digest"] == function_key())
                .expect("the function");
            function["body"] = json!({"term": "aggregate", "members": [{
                "term": "binding", "name": "parameters", "value": {
                    "term": "aggregate", "members": parameter_keys.iter().map(|key| reference_to(key)).collect::<Vec<_>>()
                }
            }]});
        }
    })
}

/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_a_signature_type_that_is_declared_or_model_owned_refuses() {
    let (set_key, set_node) = collection("set", &record_key(), &[&record_key()]);
    let tuple_key = structural_key(
        "composite_type",
        "tuple",
        Some(&boolean_key()),
        &json!({"term": "aggregate", "members": []}),
    );
    let model = family_key("1515");
    let (reference_key, reference_node) = collection("reference", &model, &[&model]);
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
                nodes.push(set_node);
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
                nodes.push(reference_node);
            }),
        ),
    ];
    for (name, dependency) in cases {
        let package = calling(&dependency, &dependency.function_key);
        let refusal = refused(read(&package, &dependency));
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped, "{name}");
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            "{name}"
        );
        assert_eq!(
            refusal.loci[0].path.clone(),
            Some(pointer(&call_path(&package, "/arguments/0"))),
            "{name}"
        );
    }
    // The same shapes over package-independent types admit: a set of the
    // bounded integer.
    let (ok_key, ok_node) = collection("set", &integer_range_key(), &[&integer_range_key()]);
    let ok = signature(&[&ok_key], &boolean_key(), |nodes| {
        nodes.push(ok_node);
    });
    let package = calling(&ok, &ok.function_key);
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
fn parameter(semantic_type: &str, text: &str, integer: &str) -> Value {
    let literal = |type_key: &str, kind: &str, value: &str| json!({"term": "literal", "type": node_id(type_key), "value_kind": kind, "value": value});
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": literal(text, "text", "x")},
        {"term": "binding", "name": "level", "value": literal(integer, "integer", "0")},
    ]});
    let key = structural_key("value", "parameter", Some(semantic_type), &body);
    let mut node = node(&key, "value", "parameter", semantic_type, &[], None);
    node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    with_body(node, body)
}

/// The dependency function with a parameter node outside its declared
/// `parameters` binding.
fn unbound_parameter_dependency(listed: bool, referenced: bool) -> Dependency {
    // The fixture already holds the anonymous `Text` and `Integer` nodes.
    let (text, integer) = (family_key("a1a1"), family_key("a3a3"));
    let parameter = parameter(&record_key(), &text, &integer);
    let parameter_key = parameter["node_id"]["digest"]
        .as_str()
        .expect("parameter key")
        .to_owned();
    build_dependency(|nodes| {
        nodes.push(node(
            &record_key(),
            "composite_type",
            "record",
            &boolean_key(),
            &[&boolean_key()],
            Some(&["R"]),
        ));
        nodes.push(parameter);
        let function = nodes
            .iter_mut()
            .find(|node| node["node_id"]["digest"] == function_key())
            .expect("the function");
        if referenced {
            function["body"] = json!({
                "term": "aggregate", "members": [reference_to(&parameter_key)],
            });
        }
        if listed {
            function["dependencies"] = json!([node_id(&parameter_key)]);
        }
    })
}

#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn unbound_parameter_type_is_outside_the_signature() {
    for (listed, referenced) in [(true, false), (false, true), (true, true)] {
        let dependency = unbound_parameter_dependency(listed, referenced);
        let package = calling(&dependency, &dependency.function_key);
        assert!(
            matches!(
                read(&package, &dependency),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "a parameter node outside the binding is not a signature parameter"
        );
    }
}

#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn signature_type_closure_follows_dependencies_only() {
    let by_type = structural_key(
        "composite_type",
        "tuple",
        Some(&record_key()),
        &json!({"term": "aggregate", "members": []}),
    );
    // A set no `dependencies` entry ties to the record: its body references
    // it.
    let (by_body, by_body_node) = collection("set", &record_key(), &[]);
    let body = signature(&[&by_body], &boolean_key(), |nodes| {
        nodes.push(by_body_node);
    });
    // A tuple (a form whose key the reader does not derive) whose semantic
    // type is the record: a derived-shape node is its own semantic type, so
    // only another form carries the semantic-type route.
    let typed = signature(&[&by_type], &boolean_key(), |nodes| {
        nodes.push(node(
            &by_type,
            "composite_type",
            "tuple",
            &record_key(),
            &[],
            None,
        ));
    });
    for (name, dependency) in [("body reference", body), ("semantic type", typed)] {
        let package = calling(&dependency, &dependency.function_key);
        assert!(
            matches!(
                read(&package, &dependency),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "a signature type's {name} is outside its dependency closure"
        );
    }
}

#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn dependency_function_body_reference_outside_signature_is_eligible() {
    let dependency = build_dependency(|nodes| {
        nodes.push(node(
            &record_key(),
            "composite_type",
            "record",
            &boolean_key(),
            &[&boolean_key()],
            Some(&["R"]),
        ));
        let function = nodes
            .iter_mut()
            .find(|node| node["node_id"]["digest"] == function_key())
            .expect("the function");
        function["dependencies"] = json!([node_id(&record_key())]);
        function["body"] = json!({
            "term": "aggregate", "members": [reference_to(&record_key())]
        });
    });
    let package = calling(&dependency, &dependency.function_key);
    let result = read(&package, &dependency);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "a body-only declared record is outside the dependency function signature: {result:?}"
    );
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
    mint_ungrouped_structural_keys(&mut package);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let function_key = declared_function_key(&package);
    let (digest, admitted) = admitted_dependency(&package);
    Dependency {
        package: admitted,
        digest,
        function_key,
    }
}

fn node_key(package: &Value, position: usize) -> String {
    package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("key")
        .to_owned()
}

/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_a_quantity_over_a_declared_unit_refuses() {
    let recorded = v2_nominal();
    // The recorded package's unit is `Example::Metre`; the function returns
    // it.
    let unit = node_key(&recorded, 2);
    let dependency = nominal_dependency(recorded, &unit, |_| {});
    let package = calling(&dependency, &dependency.function_key);
    expect_refusal(
        &refused(read(&package, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&package, "/arguments/0"),
    );
}

/// ACs: FR-038-AC-38
#[trace("TC-048", "FR-038-AC-38")]
#[test]
fn tc_048_a_type_owned_by_a_domain_package_refuses_without_a_declaration() {
    // The recorded nominal package with its enum owned by a domain package
    // and carrying no `declaration`: only its owner makes it
    // package-dependent.
    let owner = json!({"kind": "model", "identity": "test/orders",
        "node": "ix://test/orders/Status"});
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
    select_model(&mut owned, "test/orders");
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
    let package = calling(&dependency, &dependency.function_key);
    expect_refusal(
        &refused(read(&package, &dependency)),
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &call_path(&package, "/arguments/0"),
    );
}

/// A binding's value is no callee, and a call nested in another term, which
/// would have its own callee, is outside the body grammar: an application
/// stands only at a body root (FR-038-AC-114).
/// ACs: FR-038-AC-36, FR-038-AC-114
#[trace("TC-048", "FR-038-AC-36", "FR-038-AC-114")]
#[test]
fn tc_048_a_binding_value_refuses_and_a_nested_call_is_outside_the_grammar() {
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
    // A nested `function.call` is no callee position at all: it is refused
    // `malformed_wire` at the nested application, whatever its arguments.
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
    expect_refusal(
        &refused(read(&nested, &dependency)),
        CheckedPackageRefusalCode::MalformedWire,
        None,
        &call_path(&nested, "/arguments/1"),
    );
    let misplaced = importing_with(entries(), |arguments| {
        arguments[0] = term.clone();
        arguments.push(nested_call(vec![term.clone(), term.clone()]));
    });
    expect_refusal(
        &refused(read(&misplaced, &dependency)),
        CheckedPackageRefusalCode::MalformedWire,
        None,
        &call_path(&misplaced, "/arguments/1"),
    );
}
