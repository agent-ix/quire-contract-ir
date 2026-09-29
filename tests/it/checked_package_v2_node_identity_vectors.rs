// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-057: QSpec's published `node-identity-vectors.json` through the V2
//! reader, array by array: `vectors` and `operation_vectors` re-derive their
//! recorded keys and their nodes admit; every `invalid_mutations` entry,
//! patched and keyed under its `retained_sha256`, refuses with its recorded
//! code; every `operation_mutations` entry refuses with its recorded code and
//! cause, a `stale_key` one keyed under its `retained_sha256` and every other
//! one keyed by its patched preimage (the key it refuses under when retained
//! is itself refused first).
//!
//! Read at run time from the checkout `QSPEC_DIR` names; nothing of QSpec is
//! copied into this repository. The test skips (and passes) when `QSPEC_DIR`
//! is unset; `make qspec-vectors` requires it. The `frame_mutations` array is
//! TC-056's (`checked_package_v2_frame_entries.rs`).

use crate::support::checked_package::{
    canonical, domain_package_digest, domain_package_document, evidence_for, nominal_package,
    pointer, refresh_identity, sha256_hex,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

use CheckedPackageRefusalCause as Cause;

/// The entry counts QSpec publishes in `node-identity-vectors.json`.
const NOMINAL_VECTORS: usize = 17;
const OPERATION_VECTORS: usize = 21;
const INVALID_MUTATIONS: usize = 13;
const OPERATION_MUTATIONS: usize = 24;
use CheckedPackageRefusalCode as Code;

const VECTORS: &str = "proposals/checked-package-v2/node-identity-vectors.json";
const OPERATION_PACKAGE: &str =
    "proposals/checked-package-v2/fixtures/positive-operation-identities.json";

/// A file of the QSpec checkout `QSPEC_DIR` names, or `None` when unset.
fn qspec(relative: &str) -> Option<Value> {
    let root = std::env::var_os("QSPEC_DIR")?;
    let path = std::path::Path::new(&root).join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    Some(serde_json::from_str(&text).expect("QSpec JSON"))
}

fn text<'v>(value: &'v Value, member: &str) -> &'v str {
    value[member]
        .as_str()
        .unwrap_or_else(|| panic!("{member} is a string in {value}"))
}

fn array<'v>(value: &'v Value, member: &str) -> &'v [Value] {
    value[member]
        .as_array()
        .unwrap_or_else(|| panic!("{member} is an array"))
}

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    )
}

fn admits(case: &str, value: &Value) {
    match read(value) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("{case} admits, read {other:?}"),
    }
}

fn refused(case: &str, value: &Value) -> CheckedPackageRefusal {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{case} refuses, read {other:?}"),
    }
}

fn code(name: &str) -> Code {
    match name {
        "invalid_semantic_graph" => Code::InvalidSemanticGraph,
        "invalid_package" => Code::InvalidPackage,
        "ill_typed" => Code::IllTyped,
        other => panic!("vector code {other} is not mapped"),
    }
}

fn cause(name: &str) -> Cause {
    match name {
        "stale-node-key" => Cause::StaleNodeKey,
        "unknown-operation" => Cause::UnknownOperation,
        "operation-class-mismatch" => Cause::OperationClassMismatch,
        "operation-law-missing" => Cause::OperationLawMissing,
        "operation-law-mismatch" => Cause::OperationLawMismatch,
        "operation-law-unselected" => Cause::OperationLawUnselected,
        "operation-mode-mismatch" => Cause::OperationModeMismatch,
        "operation-mode-type-mismatch" => Cause::OperationModeTypeMismatch,
        "operation-member-mismatch" => Cause::OperationMemberMismatch,
        "operator-ineligible" => Cause::OperatorIneligible,
        other => panic!("vector cause {other} is not mapped"),
    }
}

/// Applies one RFC 6902 operation (`add`, `remove`, `replace`, `move`).
fn apply(target: &mut Value, operation: &Value) {
    fn split(path: &str) -> (&str, String) {
        let (parent, last) = path.rsplit_once('/').expect("a pointer below the root");
        (parent, last.replace("~1", "/").replace("~0", "~"))
    }
    fn remove(target: &mut Value, path: &str) -> Value {
        let (parent, last) = split(path);
        match target.pointer_mut(parent).expect("the parent exists") {
            Value::Object(object) => object.remove(&last).expect("the member exists"),
            Value::Array(items) => items.remove(last.parse().expect("an index")),
            other => panic!("cannot remove from {other}"),
        }
    }
    fn add(target: &mut Value, path: &str, value: Value) {
        let (parent, last) = split(path);
        match target.pointer_mut(parent).expect("the parent exists") {
            Value::Object(object) => {
                object.insert(last, value);
            }
            Value::Array(items) if last == "-" => items.push(value),
            Value::Array(items) => items.insert(last.parse().expect("an index"), value),
            other => panic!("cannot add to {other}"),
        }
    }
    let path = text(operation, "path");
    match text(operation, "op") {
        "replace" => {
            *target.pointer_mut(path).expect("the member exists") = operation["value"].clone();
        }
        "remove" => {
            remove(target, path);
        }
        "add" => add(target, path, operation["value"].clone()),
        "move" => {
            let value = remove(target, text(operation, "from"));
            add(target, path, value);
        }
        other => panic!("patch op {other} is not mapped"),
    }
}

fn patched(preimage: &Value, patch: &[Value]) -> Value {
    let mut candidate = preimage.clone();
    for operation in patch {
        apply(&mut candidate, operation);
    }
    candidate
}

/// The vector named `name` in `list`.
fn vector<'v>(list: &'v [Value], name: &str) -> &'v Value {
    list.iter()
        .find(|vector| vector["name"] == name)
        .unwrap_or_else(|| panic!("{name} is a vector"))
}

/// A nominal package over `members`, whose lock selects every owner the
/// published vectors name: the `agent-ix`/`example-model` definition and
/// the `acme/orders` domain package at `model_version`.
fn nominal_vector_package(members: &[(Value, String)], model_version: &str) -> Value {
    let mut package = nominal_package(members);
    let lock = &mut package["lock"];
    let mut definition = lock["definition_selections"][0].clone();
    definition["authority"] = json!("agent-ix");
    definition["identity"] = json!("example-model");
    lock["definition_selections"]
        .as_array_mut()
        .expect("definition selections")
        .push(definition);
    let document = domain_package_document("acme/orders", model_version, Vec::new());
    lock["model_selections"] = json!([{
        "identity": "acme/orders",
        "version": model_version,
        "digest_domain": "sha256-jcs",
        "digest": domain_package_digest(&document),
    }]);
    refresh_identity(&mut package);
    package
}

/// The model version the vector's owner names, `1.0.0` when it names none.
fn model_version(preimage: &Value) -> &str {
    preimage["owner"]["version"].as_str().unwrap_or("1.0.0")
}

/// Every node key `body` references, and every operation member
/// declaration, unique and ascending: the application-node dependency join.
fn dependency_join(body: &Value) -> Vec<Value> {
    let mut keys = Vec::new();
    let mut pending = vec![body];
    while let Some(term) = pending.pop() {
        match term["term"].as_str() {
            Some("reference") => keys.push(term["target"].clone()),
            Some("application") => {
                if term["operation"]["member"]["declaration"].is_object() {
                    keys.push(term["operation"]["member"]["declaration"].clone());
                }
                pending.extend(term["arguments"].as_array().into_iter().flatten());
            }
            Some("aggregate") => pending.extend(term["members"].as_array().into_iter().flatten()),
            Some("binding") => pending.push(&term["value"]),
            _ => {}
        }
    }
    keys.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
    keys.dedup();
    keys
}

/// Writes an application-node preimage's members into `node`.
fn write_preimage(node: &mut Value, preimage: &Value) {
    for member in ["node_tag", "semantic_form", "semantic_type", "body"] {
        node[member] = preimage[member].clone();
    }
    match preimage["declaration"].clone() {
        Value::Null => {
            node.as_object_mut().expect("node").remove("declaration");
        }
        declaration => node["declaration"] = declaration,
    }
    node["dependencies"] = Value::Array(dependency_join(&preimage["body"]));
}

/// Renames the node key `stale` to `fresh` everywhere in `value`.
fn rename(value: &mut Value, stale: &str, fresh: &str) {
    match value {
        Value::String(text) if text == stale => *text = fresh.to_owned(),
        Value::Array(items) => items.iter_mut().for_each(|item| rename(item, stale, fresh)),
        Value::Object(members) => members
            .values_mut()
            .for_each(|member| rename(member, stale, fresh)),
        _ => {}
    }
}

fn node_position(package: &Value, digest: &str) -> Option<usize> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"]["digest"] == digest)
}

/// Tracing: TC-057
/// ACs: FR-038-AC-40
#[trace("TC-057", "FR-038-AC-40")]
#[test]
fn qspec_node_identity_vectors() {
    let Some(published) = qspec(VECTORS) else {
        println!("skipped: QSPEC_DIR not set");
        return;
    };

    // `vectors`: each key re-derives from its preimage, and the nodes admit
    // in one package per selected domain package version (one lock selects
    // one version of a model identity).
    let vectors = array(&published, "vectors");
    for vector in vectors {
        assert_eq!(
            sha256_hex(&canonical(&vector["preimage"])),
            text(vector, "sha256"),
            "{}",
            text(vector, "name")
        );
    }
    let versions: std::collections::BTreeSet<&str> = vectors
        .iter()
        .map(|vector| model_version(&vector["preimage"]))
        .collect();
    let mut placed = 0_usize;
    for version in &versions {
        let members: Vec<(Value, String)> = vectors
            .iter()
            .filter(|vector| model_version(&vector["preimage"]) == *version)
            .map(|vector| {
                (
                    vector["preimage"].clone(),
                    text(vector, "sha256").to_owned(),
                )
            })
            .collect();
        placed += members.len();
        admits(
            &format!("the nominal vectors of model version {version}"),
            &nominal_vector_package(&members, version),
        );
    }
    assert_eq!(placed, vectors.len());

    // `invalid_mutations`: the patched preimage keyed under its retained
    // key, in the package of its base vector.
    let invalid = array(&published, "invalid_mutations");
    for mutation in invalid {
        let name = text(mutation, "name");
        let base = vector(vectors, text(mutation, "base"));
        let version = model_version(&base["preimage"]);
        let candidate = patched(&base["preimage"], array(mutation, "patch"));
        let members: Vec<(Value, String)> = vectors
            .iter()
            .filter(|vector| model_version(&vector["preimage"]) == version)
            .map(|vector| {
                if vector["name"] == base["name"] {
                    (
                        candidate.clone(),
                        text(mutation, "retained_sha256").to_owned(),
                    )
                } else {
                    (
                        vector["preimage"].clone(),
                        text(vector, "sha256").to_owned(),
                    )
                }
            })
            .collect();
        let refusal = refused(name, &nominal_vector_package(&members, version));
        assert_eq!(
            refusal.code,
            code(text(mutation, "expected_code")),
            "{name}: {refusal:?}"
        );
    }

    // `operation_vectors`: each key re-derives from its preimage; the
    // published operation package carries the node each names, with exactly
    // the preimage's members, and admits.
    let base_package = qspec(OPERATION_PACKAGE).expect("QSPEC_DIR is set");
    let operations = array(&published, "operation_vectors");
    let mut carried = 0_usize;
    for vector in operations {
        let name = text(vector, "name");
        let preimage = &vector["preimage"];
        assert_eq!(
            sha256_hex(&canonical(preimage)),
            text(vector, "sha256"),
            "{name}"
        );
        let Some(position) = node_position(&base_package, text(vector, "sha256")) else {
            continue;
        };
        let node = &base_package["semantic_graph"]["nodes"][position];
        for member in ["node_tag", "semantic_form", "semantic_type", "body"] {
            assert_eq!(node[member], preimage[member], "{name}: {member}");
        }
        assert_eq!(
            node.get("declaration").unwrap_or(&Value::Null),
            &preimage["declaration"],
            "{name}"
        );
        carried += 1;
    }
    admits("the published operation package", &base_package);

    // `operation_mutations`: the base vector's node in the operation package
    // takes the patched preimage, keyed under the retained key; a stale-key
    // mutation's patched preimage is the vector it names in `rekeyed_as`.
    let mutations = array(&published, "operation_mutations");
    let mut rekeyed = std::collections::BTreeSet::new();
    for mutation in mutations {
        let name = text(mutation, "name");
        let base = vector(operations, text(mutation, "base"));
        let candidate = patched(&base["preimage"], array(mutation, "patch"));
        if let Some(target) = mutation["rekeyed_as"].as_str() {
            assert_eq!(
                sha256_hex(&canonical(&candidate)),
                text(vector(operations, target), "sha256"),
                "{name} rekeys as {target}"
            );
            rekeyed.insert(target);
        }
        let mut package = base_package.clone();
        let position = node_position(&package, text(base, "sha256"))
            .unwrap_or_else(|| panic!("{name}: the package carries its base"));
        assert_eq!(
            text(base, "sha256"),
            text(mutation, "retained_sha256"),
            "{name} retains its base key"
        );
        write_preimage(
            &mut package["semantic_graph"]["nodes"][position],
            &candidate,
        );
        let mut retained = package.clone();
        refresh_identity(&mut retained);
        if text(mutation, "kind") != "stale_key" {
            // Retaining the base key is itself refused as a stale key at the
            // node's own `node_id`, before any operation check ...
            let refusal = refused(name, &retained);
            assert_eq!(
                (refusal.code, refusal.cause, refusal.path.clone()),
                (
                    Code::InvalidPackage,
                    Some(Cause::StaleNodeKey),
                    Some(pointer(&format!(
                        "/semantic_graph/nodes/{position}/node_id"
                    ))),
                ),
                "{name} retaining its key: {refusal:?}"
            );
            // ... so the operation refusal is read with the node keyed by
            // its patched preimage.
            rename(
                &mut package,
                text(base, "sha256"),
                &sha256_hex(&canonical(&candidate)),
            );
        }
        refresh_identity(&mut package);
        let refusal = refused(name, &package);
        assert_eq!(
            refusal.code,
            code(text(mutation, "expected_code")),
            "{name}: {refusal:?}"
        );
        assert_eq!(
            refusal.cause,
            Some(cause(text(mutation, "expected_cause"))),
            "{name}: {refusal:?}"
        );
    }
    // The operation vectors the published package does not carry join it as
    // nodes of their own, with the laws they name selected, and it admits.
    let mut every = base_package.clone();
    let template = every["semantic_graph"]["nodes"][0].clone();
    let mut added = 0_usize;
    for vector in operations {
        let key = text(vector, "sha256");
        if node_position(&every, key).is_some() {
            continue;
        }
        let mut node = template.clone();
        write_preimage(&mut node, &vector["preimage"]);
        node["node_id"]["digest"] = json!(key);
        for law in vector["preimage"]["body"]["operation"]["laws"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let selections = every["lock"]["definition_selections"]
                .as_array_mut()
                .expect("definition selections");
            if !selections.contains(&law["definition"]) {
                selections.push(law["definition"].clone());
            }
        }
        let mut entry = every["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .find(|entry| entry["node_id"] == template["node_id"])
            .expect("the template's source map entry")
            .clone();
        entry["node_id"] = node["node_id"].clone();
        every["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(entry);
        every["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        added += 1;
    }
    refresh_identity(&mut every);
    admits(
        "the published operation package with every operation vector",
        &every,
    );
    assert_eq!(carried + added, operations.len());
    assert!(rekeyed.iter().all(|name| node_position(
        &every,
        text(vector(operations, name), "sha256")
    )
    .is_some()));

    // The counts QSpec publishes per array; an empty or truncated array
    // fails here rather than replaying nothing.
    for (array_name, replayed, published) in [
        ("vectors", vectors.len(), NOMINAL_VECTORS),
        ("operation_vectors", operations.len(), OPERATION_VECTORS),
        ("invalid_mutations", invalid.len(), INVALID_MUTATIONS),
        ("operation_mutations", mutations.len(), OPERATION_MUTATIONS),
    ] {
        assert!(
            replayed >= published,
            "{replayed} {array_name} replayed, expected at least {published}"
        );
    }
    println!(
        "conformance: node-identity-vectors {} vectors + {} operation_vectors ({carried} carried) + {} invalid_mutations + {} operation_mutations",
        vectors.len(),
        operations.len(),
        invalid.len(),
        mutations.len()
    );
}
