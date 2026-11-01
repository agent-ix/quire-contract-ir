// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Shared helpers for the I04 CheckedPackage integration tests.
//!
//! The package documents these helpers used to read were copied from a
//! private repository and were removed when this repository's contents were
//! contained; see AGE-1961.
//!
//! [`v2_all_families`], [`v2_nominal`] and [`positive_operation_identities`]
//! are regenerated below: every document they return is *built* by
//! [`build_v2_all_families`]/[`nominal_package`]/[`build_operation_identities`]
//! from this crate's own public vocabulary (`CheckedNodeTag`, the
//! `quire.application-node/v1` key-derivation formula the reader implements in
//! `v2/operations.rs`, and the nominal identity preimages `v2/identity.rs`
//! implements) — SHA-256/RFC-8785 canonicalization done with the same
//! `sha256_hex`/`canonical` helpers every other test in this module already
//! used, never a copy of bytes from anywhere else. The node-identity-vectors
//! conformance oracle is the one piece that stays unavailable: it was an
//! *independent* oracle (what an outside producer computed), so regenerating
//! it from this crate would make every assertion against it tautological —
//! see AGE-1961. Every test that read it was removed rather than rewritten.

#![allow(dead_code)] // Each test binary uses a different subset of these helpers.

use quire_contract_model::{
    CheckedNodeId, CheckedNodePathSegment, CheckedPackageEvidence, CheckedPackageIncomplete,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, JsonPointer,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

pub const COMPLETE_VALUE_FEATURE: &str = "quire.value.complete/v1";
pub const FIXTURE_PACKAGE: &str = "test/families";
const FAMILY_MODEL_IDENTITY: &str = "test/families";
const FAMILY_OBJECT: &str = "ix://test/families/Account";
const FAMILY_PROCESS: &str = "ix://test/families/Flow";
const FAMILY_INTERFACE: &str = "ix://test/families/Interface";
const FAMILY_RELATIONSHIP: &str = "ix://test/families/relationship/Account-balance-Account";

pub fn family_model_document() -> Value {
    let fields = ["balance", "total"]
        .into_iter()
        .map(|name| {
            json!({
                "identity": format!("{FAMILY_OBJECT}/{name}"),
                "name": name, "typeRef": "ix://quire/native/Integer",
                "presence": "required", "nullable": false, "defaultKind": "none",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
            })
        })
        .collect::<Vec<_>>();
    let operations = ["deposit", "withdraw"]
        .into_iter()
        .map(|name| json!({"identity": format!("{FAMILY_OBJECT}/{name}"), "params": []}))
        .collect::<Vec<_>>();
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": FAMILY_MODEL_IDENTITY, "version": "1"},
        "constructs": [{
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }, {
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "interface"},
            "construct": {"meaning": "quire.meaning.systems.interface/v1"},
        }, {
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "process"},
            "construct": {"meaning": "quire.meaning.model.process/v1"},
        }],
        "types": [{
            "identity": FAMILY_OBJECT, "displayName": "Account",
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "entity"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [],
            "fields": fields,
            "operations": operations,
            "relationships": [{
                "identity": FAMILY_RELATIONSHIP,
                "category": "structural", "composite": false, "direction": "bidirectional",
                "sourceEnd": {"type": FAMILY_OBJECT, "role": "balanceRel",
                    "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}},
                "targetEnd": {"type": FAMILY_OBJECT,
                    "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}},
                "origin": {"source": {"sourceIdentity": FAMILY_OBJECT,
                    "path": "models/Account.md", "startLine": 1, "startColumn": 1}},
            }],
        }, {
            "identity": FAMILY_INTERFACE, "displayName": "Interface",
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "interface"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [], "fields": [], "operations": [],
        }, {
            "identity": FAMILY_PROCESS, "displayName": "Flow",
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "process"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [],
            "fields": [{
                "identity": format!("{FAMILY_PROCESS}/balance"), "name": "balance",
                "typeRef": "ix://quire/native/Integer",
                "presence": "required", "nullable": false, "defaultKind": "none",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}
            }],
            "operations": [],
            "identityFields": [format!("{FAMILY_PROCESS}/balance")],
            "steps": [{
                "identity": format!("{FAMILY_PROCESS}/advance"), "name": "advance",
                "stepKind": "command", "consumes": [], "emits": []
            }],
        }],
    })
}

/// Adds the selected Account declaration as a valid `Reference` target to a
/// synthetic package that otherwise contains only anonymous or nominal nodes.
pub fn add_family_reference_target(package: &mut Value) -> String {
    let key = family_key("1515");
    let selected = v2_all_families();
    select_model(package, FAMILY_MODEL_IDENTITY);
    let mut declaration = selected["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == node_id(&key))
        .expect("selected Account")
        .clone();
    declaration["dependencies"] = json!([]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(declaration);
    key
}

/// The exact minimum `work` `CheckedPackageV2::read` charges to admit
/// `v2_all_families()`. Measured by binary search directly against the real
/// reader — the same technique
/// `tc_048_shipped_default_read_limits_are_exact_and_finite` uses — rather
/// than hand-derived from the validation stages' documented charges: a
/// hand-derived figure is exactly the kind of number that silently drifts
/// from what the reader actually charges the next time this generator's
/// fixture changes (AGE-1961 exists because the
/// previous fixture tree could not be regenerated when that happened).
/// Shared between `complete_v1_checked_package` and `checked_package_v2_reader`
/// so a change to `build_v2_all_families` cannot silently move the boundary
/// in only one of them.
pub fn all_families_read_work() -> u64 {
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let bounded = CheckedPackageReadLimits::bounded();
    let (mut lo, mut hi) = (0_u64, bounded.work);
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        let mut limits = bounded;
        limits.work = mid;
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => hi = mid,
            _ => lo = mid,
        }
    }
    hi
}

pub fn v2_all_families() -> Value {
    build_v2_all_families()
}

pub fn v2_nominal() -> Value {
    nominal_package(&nominal_fixture_members())
}

/// [`v2_all_families`] with a chain of `length` further `expression`/`reference`
/// nodes, each referencing the one before it: the first the family's own
/// reference node, and the last, whose key is returned, the end of an
/// expression `length` nodes deep. Each node costs one node, one edge and one
/// occurrence, so a limit sized for the node count decides nothing else, and the
/// body grammar keeps the document's JSON depth at that of a one-node chain
/// (FR-038-AC-117, merged QSpec FR-322-AC-41).
///
/// Returned as the canonical document's bytes, built as text one node at a time
/// and spliced into the family's document at three markers, with `package_id`
/// derived over the spliced identity preimage: no tree of the whole document is
/// ever held, which would be many times the size of its text.
pub fn v2_reference_chain(length: usize) -> (Vec<u8>, String) {
    const NODES: &str = "\u{1}nodes";
    const SOURCE_MAP: &str = "\u{1}source_map";
    let boolean = family_key("aaaa");
    let mut previous = family_key("eeee");
    let mut package = v2_all_families();
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let source = fixture_source();
    let (mut nodes, mut source_map) = (Vec::new(), Vec::new());
    for index in 0..length {
        let body = json!({"term": "reference", "target": node_id(&previous)});
        let key = format!("expression/reference/{index}");
        let node = plain_node(
            &key,
            "expression",
            "reference",
            &boolean,
            &[previous.as_str()],
            body,
        );
        nodes.push(serde_json::to_string(&node).expect("node"));
        let position = base + index;
        source_map.push(
            serde_json::to_string(&json!({
                "node_id": node_id(&key), "role": "generated", "ordinal": 0,
                "regions": [{"source": source, "start": position, "end": position + 1}],
            }))
            .expect("source map entry"),
        );
        previous = key;
    }
    let marker = |package: &mut Value, pointer: &str, text: &str| {
        package
            .pointer_mut(pointer)
            .and_then(Value::as_array_mut)
            .expect("an array")
            .push(json!(text));
    };
    marker(&mut package, "/semantic_graph/nodes", NODES);
    marker(&mut package, "/source_map", SOURCE_MAP);
    let splice = |text: String, marker: &str, entries: &[String]| {
        let quoted = serde_json::to_string(marker).expect("marker");
        text.replace(&quoted, &entries.join(","))
    };
    let text = String::from_utf8(canonical(&package)).expect("utf-8");
    let text = splice(text, NODES, &nodes);
    let text = splice(text, SOURCE_MAP, &source_map);
    (text.into_bytes(), previous)
}

/// A `quire.checked-package/v2` document exercising `declaration`,
/// `literal.type` and `application.operation`/`application.result_type` —
/// the shape `tc_048_deleting_a_declared_wire_member_refuses_before_the_projection_compare`
/// needs a node carrying each of, so it can delete one member at a time and
/// assert the closed-schema refusal that member's absence produces.
pub fn positive_operation_identities() -> Value {
    build_operation_identities()
}

/// RFC 8785 bytes for the ASCII, integer-only fixtures (sorted members).
pub fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("canonical test JSON")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn node_id(path_label: &str) -> Value {
    let model_path = match path_label {
        "fixture/1515" => Some(json!(["model", "object_type", FAMILY_OBJECT])),
        "fixture/1616" => Some(json!(["model", "process", FAMILY_PROCESS])),
        "fixture/2020" => Some(json!(["relation", "relationship", FAMILY_RELATIONSHIP])),
        _ => None,
    };
    if let Some(path) = model_path {
        return json!({"package": FAMILY_MODEL_IDENTITY, "path": path, "ordinal": 0});
    }
    let path = path_label.strip_prefix("qualified:").map_or_else(
        || vec![path_label],
        |qualified| qualified.split('/').collect(),
    );
    json!({"package": FIXTURE_PACKAGE, "path": path, "ordinal": 0})
}

/// A structural fixture identity with an explicit package, path and ordinal.
pub fn structural_id(package: &str, path: Vec<Value>, ordinal: u64) -> Value {
    json!({"package": package, "path": path, "ordinal": ordinal})
}

/// Builds a structural expected key from a test's authored pre-index identity.
/// This is a fixture constructor, never a reader for the V2 wire.
pub fn structural_fixture_id(value: &Value) -> CheckedNodeId {
    let path = value["path"]
        .as_array()
        .expect("fixture structural path")
        .iter()
        .map(|segment| match segment {
            Value::String(name) => CheckedNodePathSegment::Name(name.as_str().into()),
            Value::Object(_) => {
                CheckedNodePathSegment::Node(std::sync::Arc::new(structural_fixture_id(segment)))
            }
            _ => panic!("fixture structural path segment"),
        })
        .collect();
    CheckedNodeId {
        package: value["package"].as_str().expect("fixture package").into(),
        path,
        ordinal: value["ordinal"].as_u64().expect("fixture ordinal"),
    }
}

/// Rewrites authored structural fixture identities into the V2 graph-local
/// index wire. Every nested operand must name a node in this document's graph;
/// a missing fixture node is an error in the test construction.
pub fn index_fixture_node_ids(package: &mut Value) {
    let ids = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("semantic graph nodes")
        .iter()
        .map(|node| node["node_id"].clone())
        .collect::<Vec<_>>();

    fn rewrite(value: &mut Value, ids: &[Value]) {
        if let Some(path) = value
            .as_object_mut()
            .filter(|object| {
                object.len() == 3
                    && object.contains_key("package")
                    && object.contains_key("path")
                    && object.contains_key("ordinal")
            })
            .and_then(|object| object.get_mut("path"))
            .and_then(Value::as_array_mut)
        {
            for segment in path {
                if segment.is_object() {
                    if segment.as_object().is_some_and(|object| {
                        object.len() == 1 && object.get("node").and_then(Value::as_u64).is_some()
                    }) {
                        continue;
                    }
                    let index = ids
                        .iter()
                        .position(|id| id == segment)
                        .expect("nested fixture operand must be a graph node");
                    *segment = json!({"node": index});
                }
            }
        }
        match value {
            Value::Array(items) => {
                for item in items {
                    rewrite(item, ids);
                }
            }
            Value::Object(object) => {
                for member in object.values_mut() {
                    rewrite(member, ids);
                }
            }
            _ => {}
        }
    }

    rewrite(package, &ids);
    refresh_identity(package);
}

/// Renames one structural identity throughout a synthetic package, including
/// body references, dependency edges, nested paths and source-map entries.
pub fn reassign_node_id(package: &mut Value, old: &Value, fresh: &Value) {
    if package == old {
        *package = fresh.clone();
        return;
    }
    match package {
        Value::Array(items) => {
            for item in items {
                reassign_node_id(item, old, fresh);
            }
        }
        Value::Object(members) => {
            for item in members.values_mut() {
                reassign_node_id(item, old, fresh);
            }
        }
        _ => {}
    }
}

/// Renames a set of exact structural identities in one pass over a synthetic
/// package. A replacement is never revisited, so mappings remain simultaneous.
pub fn reassign_node_ids(
    package: &mut Value,
    replacements: &BTreeMap<CheckedNodeId, CheckedNodeId>,
) {
    match package {
        Value::Object(members) => {
            if members.len() == 3
                && members.contains_key("package")
                && members.contains_key("path")
                && members.contains_key("ordinal")
            {
                let candidate =
                    serde_json::from_value::<CheckedNodeId>(Value::Object(members.clone()));
                if let Ok(id) = candidate {
                    if let Some(fresh) = replacements.get(&id) {
                        *package = serde_json::to_value(fresh).expect("typed node id serializes");
                        return;
                    }
                }
            }
            for member in members.values_mut() {
                reassign_node_ids(member, replacements);
            }
        }
        Value::Array(items) => {
            for item in items {
                reassign_node_ids(item, replacements);
            }
        }
        _ => {}
    }
}

/// The source owner attached to a declared structural fixture node.
pub fn source_owner(source: &Value) -> Value {
    json!({
        "kind": "source", "authority": source["authority"],
        "identity": source["identity"],
    })
}

/// The model owner attached to an undeclared structural fixture node.
pub fn model_owner(identity: &str, node: &str) -> Value {
    json!({"kind": "model", "identity": identity, "node": node})
}

/// Attaches the owner before a fixture projects or derives the node's key.
/// The caller keeps its existing key-derivation oracle at this seam.
pub fn owned_structural_node(mut node: Value, owner: Value) -> Value {
    node["owner"] = owner;
    node
}

pub fn typed_node_id(digest: &str) -> quire_contract_model::CheckedNodeId {
    serde_json::from_value(node_id(digest)).expect("node id")
}

/// A minimal Semantic IR 2.0.0 document for a domain package, declaring the
/// given object types.
pub fn domain_package_document(identity: &str, version: &str, types: Vec<Value>) -> Value {
    serde_json::json!({
        "contractVersion": "2.0.0",
        "package": {"identity": identity, "version": version},
        "constructs": [],
        "types": types,
    })
}

/// Evidence for a package: its domain package documents plus the complete-value feature.
pub fn evidence_for(package: &Value) -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    // Each `model_root` owner selects a domain package by identity; the
    // evidence holds the document this helper builds for that identity.
    for identity in selected_model_identities(package) {
        let document = if identity == FAMILY_MODEL_IDENTITY {
            family_model_document()
        } else {
            domain_package_document(&identity, "1", Vec::new())
        };
        evidence
            .insert_domain_package_document(identity, canonical(&document))
            .expect("domain evidence declarations agree on exact bytes");
    }
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
    evidence
}

/// Evidence for a package whose selected domain package `identity` is supplied
/// as `document` in place of the helper's default document.
pub fn evidence_with_document(
    package: &Value,
    identity: &str,
    document: &Value,
) -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    for selected in selected_model_identities(package) {
        let supplied = if selected == identity {
            document.clone()
        } else if selected == FAMILY_MODEL_IDENTITY {
            family_model_document()
        } else {
            domain_package_document(&selected, "1", Vec::new())
        };
        evidence
            .insert_domain_package_document(selected, canonical(&supplied))
            .expect("domain evidence declarations agree on exact bytes");
    }
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
    evidence
}

/// Reads `package` with only its own evidence.
fn read_admitting(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

/// A dependency package the reader admitted: its structural name and package.
pub fn admitted_dependency(package: &Value) -> (String, CheckedPackageV2) {
    match read_admitting(package) {
        CheckedPackageV2ReadResult::Admitted(admitted) => (admitted.name().to_owned(), *admitted),
        other => panic!("the dependency package admits, read {other:?}"),
    }
}

/// `read`, with `dependencies` supplied as `(identity, package)`.
pub fn read_with_dependencies(
    package: &Value,
    dependencies: &[(&str, &CheckedPackageV2)],
) -> CheckedPackageV2ReadResult {
    let mut evidence: CheckedPackageEvidence = evidence_for(package);
    for (identity, dependency) in dependencies {
        evidence
            .insert_dependency_package(*identity, (*dependency).clone())
            .expect("dependency evidence declarations agree on admitted content");
    }
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

/// Independent logical hash input; this derived value is never shipped.
pub fn identity_input(package: &Value) -> Value {
    let mut input = package["identity_preimage"].clone();
    input["identity_projection"] = Value::Array(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .map(|node| {
                let mut node = node.clone();
                node.as_object_mut().expect("node").remove("occurrences");
                node
            })
            .collect(),
    );
    input
}

/// Rebuilds the identity projection from the graph and re-derives the package id.
pub fn refresh_identity(package: &mut Value) {
    refresh_identity_projection(package);
}

/// The identity of every domain package a synthetic package selects: one per
/// `model_root` owner.
pub fn selected_model_identities(package: &Value) -> Vec<String> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|node| node["semantic_form"] == "model_root")
        .filter_map(|node| node["owner"]["identity"].as_str().map(str::to_owned))
        .collect()
}

/// Selects the domain package `identity` in a synthetic package by adding its
/// identity-named `model_root` node and source-map entry. Selecting an
/// identity twice adds nothing.
pub fn select_model(package: &mut Value, identity: &str) {
    let owner = json!({"kind": "model_root", "identity": identity});
    let exists = package["semantic_graph"]["nodes"]
        .as_array()
        .is_some_and(|nodes| {
            nodes
                .iter()
                .any(|node| node["semantic_form"] == "model_root" && node["owner"] == owner)
        });
    if exists {
        return;
    }
    let body = json!({"term": "aggregate", "members": []});
    let node_id = json!({"package": identity, "path": ["model", "model_root"], "ordinal": 0});
    let node = json!({
        "node_id": node_id, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "model", "semantic_form": "model_root",
        "semantic_type": node_id, "dependencies": [],
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "owner": owner, "body": body,
    });
    let region = package["source_map"]
        .as_array()
        .and_then(|entries| entries.first())
        .and_then(|entry| entry["regions"].as_array())
        .and_then(|regions| regions.first())
        .cloned()
        .or_else(|| {
            package["lock"]["sources"]
                .as_array()
                .and_then(|sources| sources.first())
                .map(|source| json!({"source": source, "start": 0, "end": 1}))
        });
    if let Some(region) = region {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("synthetic graph nodes")
            .push(node);
        package["source_map"]
            .as_array_mut()
            .expect("synthetic source map")
            .push(
                json!({"node_id": node_id, "role": "generated", "ordinal": 0,
                "regions": [region]}),
            );
    }
}

/// Re-derives package identity without changing its graph or source map.
pub fn refresh_identity_projection(package: &mut Value) {
    package
        .as_object_mut()
        .expect("package")
        .remove("identity_preimage");
    package
        .as_object_mut()
        .expect("package")
        .remove("package_id");
    let package_name = package["name"]
        .as_str()
        .expect("fixture package name")
        .to_owned();
    let replacements = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .filter_map(|node| {
            let old = node["node_id"].clone();
            let (owner_package, path) = match node["owner"]["kind"].as_str() {
                Some("model_root") => (
                    node["owner"]["identity"]
                        .as_str()
                        .expect("model root identity"),
                    json!(["model", "model_root"]),
                ),
                Some("model") => (
                    node["owner"]["identity"].as_str().expect("model identity"),
                    old["path"].clone(),
                ),
                _ => (
                    package_name.as_str(),
                    node.get("declaration").map_or_else(
                        || old["path"].clone(),
                        |declaration| declaration["qualified_name"].clone(),
                    ),
                ),
            };
            let fresh = json!({"package": owner_package, "path": path, "ordinal": old["ordinal"]});
            (old != fresh).then_some((old, fresh))
        })
        .collect::<Vec<_>>();
    fn replace_ids(value: &mut Value, replacements: &[(Value, Value)]) {
        if let Some((_, fresh)) = replacements.iter().find(|(old, _)| value == old) {
            *value = fresh.clone();
            return;
        }
        match value {
            Value::Array(items) => items
                .iter_mut()
                .for_each(|item| replace_ids(item, replacements)),
            Value::Object(members) => members
                .values_mut()
                .for_each(|item| replace_ids(item, replacements)),
            _ => {}
        }
    }
    replace_ids(package, &replacements);
}

/// Nesting depth counted the way the reader charges it (scalars are one).
pub fn json_depth(value: &Value) -> u64 {
    match value {
        Value::Array(values) => 1 + values.iter().map(json_depth).max().unwrap_or(0),
        Value::Object(values) => 1 + values.values().map(json_depth).max().unwrap_or(0),
        _ => 1,
    }
}

/// Recomputes each preimage's key in list order, rewriting every later
/// preimage that names a changed key. Vectors are listed in dependency order,
/// so one pass carries a rekey through every dependant.
pub fn rekey(preimages: &mut [Value], keys: &[String]) -> Vec<String> {
    let mut current = keys.to_vec();
    for position in 0..preimages.len() {
        let fresh = sha256_hex(&canonical(&preimages[position]));
        if fresh != current[position] {
            for later in preimages.iter_mut().skip(position + 1) {
                replace_digest(later, &current[position], &fresh);
            }
            current[position] = fresh;
        }
    }
    current
}

/// Renames the node `stale` to `fresh` everywhere it is named in `package`:
/// its own `node_id`, every `semantic_type`, dependency and body reference.
pub fn rename_node(package: &mut Value, stale: &str, fresh: &str) {
    replace_digest(package, stale, fresh);
}

/// Mint QSL FR-092/FR-094 keys for a builder's ungrouped structural nodes,
/// including owner-bearing and declared nodes. Each rename reaches every
/// reference in the test package; repeated passes let a node built before a
/// type it names pick up the type's real key. Nominal and grouped keys,
/// application-keyed nodes (including union values), and FR-451 abstraction
/// relation keys have separate constructors or stages.
pub fn mint_ungrouped_structural_keys(package: &mut Value) {
    let count = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    for _ in 0..=count {
        let mut changed = false;
        for position in 0..count {
            let node = package["semantic_graph"]["nodes"][position].clone();
            let tag = node["node_tag"].as_str().expect("node tag");
            let form = node["semantic_form"].as_str().expect("semantic form");
            if node.get("nominal_identity_preimage").is_some()
                || fixture_has_application_key(&node)
                || node.get("recursion_group").is_some()
                || (tag == "correspondence" && form == "abstraction_relation")
            {
                continue;
            }
            let stale = node["node_id"]["digest"].as_str().expect("node digest");
            let mut preimage = json!({
                "version": "quire.structural-node/v1",
                "node_tag": tag,
                "semantic_form": form,
                "semantic_type": (node["semantic_type"] != node["node_id"])
                    .then(|| node["semantic_type"].clone()),
                "declaration": node.get("declaration"),
                "recursion": null,
                "body": node["body"],
            });
            if let Some(owner) = node.get("owner") {
                preimage["owner"] = owner.clone();
            }
            let fresh = sha256_hex(&canonical(&preimage));
            if fresh != stale {
                rename_node(package, stale, &fresh);
                changed = true;
            }
        }
        if !changed {
            return;
        }
    }
    panic!("ungrouped fixture nodes did not reach stable structural keys");
}

/// Mint the FR-092 keys and content label of each explicit recursion group.
/// Member-local preimages replace within-group references by graph ordinals;
/// their digest array gives the label before member keys include that label.
pub fn mint_grouped_structural_keys(package: &mut Value) {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for node in nodes {
        if let Some(label) = node.get("recursion_group").and_then(Value::as_str) {
            groups.entry(label.to_owned()).or_default().push(node);
        }
    }
    for (old_label, members) in groups {
        let ids = members
            .iter()
            .enumerate()
            .map(|(ordinal, node)| {
                (
                    node["node_id"]["digest"]
                        .as_str()
                        .expect("digest")
                        .to_owned(),
                    ordinal,
                )
            })
            .collect::<HashMap<_, _>>();
        let local = members
            .iter()
            .enumerate()
            .map(|(ordinal, node)| grouped_local_key(node, &ids, ordinal, None))
            .collect::<Vec<_>>();
        let label = sha256_hex(&canonical(&json!(local)));
        for node in package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
        {
            if node["recursion_group"] == old_label {
                node["recursion_group"] = json!(label);
            }
        }
        let replacements = members
            .iter()
            .enumerate()
            .map(|(ordinal, node)| {
                (
                    node["node_id"]["digest"]
                        .as_str()
                        .expect("digest")
                        .to_owned(),
                    grouped_local_key(node, &ids, ordinal, Some(&label)),
                )
            })
            .collect::<HashMap<_, _>>();
        replace_digests(package, &replacements);
    }
}

fn replace_digests(value: &mut Value, replacements: &HashMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(fresh) = replacements.get(text) {
                *text = fresh.clone();
            }
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| replace_digests(item, replacements)),
        Value::Object(fields) => fields
            .values_mut()
            .for_each(|item| replace_digests(item, replacements)),
        _ => {}
    }
}

/// The fixture-authored FR-092 local preimage digest, optionally binding a
/// retained group label for the first structural-key pass.
pub fn grouped_local_key(
    node: &Value,
    ids: &HashMap<String, usize>,
    ordinal: usize,
    label: Option<&str>,
) -> String {
    fn rewrite(value: &Value, ids: &HashMap<String, usize>) -> Value {
        if value["term"] == "reference" {
            if let Some(ordinal) = value["target"]["digest"]
                .as_str()
                .and_then(|digest| ids.get(digest))
            {
                return json!({"term": "group_reference", "ordinal": ordinal});
            }
        }
        match value {
            Value::Array(items) => {
                Value::Array(items.iter().map(|item| rewrite(item, ids)).collect())
            }
            Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(name, value)| (name.clone(), rewrite(value, ids)))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }
    let application = fixture_has_application_key(node);
    let semantic_type = if application {
        node["semantic_type"].clone()
    } else if node["semantic_type"] == node["node_id"] {
        Value::Null
    } else if let Some(ordinal) = node["semantic_type"]["digest"]
        .as_str()
        .and_then(|digest| ids.get(digest))
    {
        json!({"term": "group_reference", "ordinal": ordinal})
    } else {
        node["semantic_type"].clone()
    };
    let mut recursion = json!({"size": ids.len(), "ordinal": ordinal});
    if !application {
        if let Some(label) = label {
            recursion["group"] = json!(label);
        }
    }
    let body = rewrite(&node["body"], ids);
    let version = if application {
        APPLICATION_NODE_VERSION
    } else {
        "quire.structural-node/v1"
    };
    let mut preimage = json!({
        "version": version,
        "node_tag": node["node_tag"],
        "semantic_form": node["semantic_form"],
        "semantic_type": semantic_type,
        "declaration": node.get("declaration"),
        "recursion": recursion,
        "body": body,
    });
    if let Some(owner) = node.get("owner") {
        if version == "quire.structural-node/v1" {
            preimage["owner"] = owner.clone();
        }
    }
    sha256_hex(&canonical(&preimage))
}

fn replace_digest(value: &mut Value, stale: &str, fresh: &str) {
    match value {
        Value::String(text) if text == stale => *text = fresh.to_owned(),
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| replace_digest(item, stale, fresh)),
        Value::Object(members) => members
            .values_mut()
            .for_each(|member| replace_digest(member, stale, fresh)),
        _ => {}
    }
}

/// Builds a canonical V2 package from nominal `(preimage, key)` pairs in the
/// recorded nominal fixture's lock, deriving each node's family, type,
/// dependencies, occurrences and body from its preimage.
pub fn nominal_package(members: &[(Value, String)]) -> Value {
    let mut package = nominal_skeleton();
    let source = package["lock"]["sources"][0].clone();
    let mut nodes = Vec::with_capacity(members.len());
    let mut source_map = Vec::with_capacity(members.len());
    let mut equivalence_ordinal = 0_u64;
    for (position, (preimage, key)) in members.iter().enumerate() {
        let id = node_id(key);
        // Every nominal form here is a named, source-declared `scalar_type`
        // (or the `enum_value` its enum declares); the schema's
        // `DeclarationTagRules`/`DeclarationOccurrenceRule` (FR-208) require
        // a `declaration` member on the former, exactly matching each node's
        // `declaration`-role occurrence below, and forbid it on the latter.
        // FR-322's `declaration-nominal-mismatch` rule pins a nominal node's
        // `declaration.qualified_name` to its own preimage's
        // `qualified_declaration`, so that member is reused verbatim rather
        // than invented.
        let declaration_name = preimage["qualified_declaration"].clone();
        let (tag, form, semantic_type, dependencies, body, declaration) =
            match preimage["version"].as_str().expect("preimage version") {
                "quire.enum-declaration-node/v1" => (
                    "scalar_type",
                    "enum",
                    id.clone(),
                    json!([]),
                    json!({
                        "term": "enum",
                        "ordered": preimage["ordered"],
                        "members": preimage["members"],
                    }),
                    Some(json!({"qualified_name": declaration_name})),
                ),
                "quire.enum-member-node/v1" => (
                    "value",
                    "enum_value",
                    preimage["declaration_node_id"].clone(),
                    json!([preimage["declaration_node_id"]]),
                    json!({
                        "term": "literal",
                        "type": preimage["declaration_node_id"],
                        "value_kind": "enum",
                        "value": preimage["case"],
                    }),
                    None,
                ),
                "quire.dimension-node/v1" => (
                    "scalar_type",
                    "dimension",
                    id.clone(),
                    Value::Array(
                        preimage["terms"]
                            .as_array()
                            .expect("terms")
                            .iter()
                            .map(|term| term["dimension_node_id"].clone())
                            .collect(),
                    ),
                    json!({"term": "dimension", "terms": preimage["terms"]}),
                    Some(json!({"qualified_name": declaration_name})),
                ),
                "quire.dimension-equivalence-node/v1" => (
                    "scalar_type",
                    "dimension_equivalence",
                    id.clone(),
                    preimage["dimension_node_ids"].clone(),
                    json!({
                        "term": "dimension_equivalence",
                        "dimension_node_ids": preimage["dimension_node_ids"],
                        "root_scale": preimage["root_scale"],
                    }),
                    None,
                ),
                "quire.unit-node/v1" => {
                    let mut dependencies = vec![preimage["dimension_node_id"].clone()];
                    if !preimage["target_unit_node_id"].is_null() {
                        dependencies.push(preimage["target_unit_node_id"].clone());
                    }
                    (
                        "scalar_type",
                        "unit",
                        preimage["dimension_node_id"].clone(),
                        Value::Array(dependencies),
                        json!({
                            "term": "unit",
                            "dimension_node_id": preimage["dimension_node_id"],
                            "target_unit_node_id": preimage["target_unit_node_id"],
                            "scale": preimage["scale"],
                            "offset": preimage["offset"],
                        }),
                        Some(json!({"qualified_name": declaration_name})),
                    )
                }
                other => panic!("unknown nominal preimage {other}"),
            };
        let is_equivalence = form == "dimension_equivalence";
        let ordinal = if is_equivalence {
            let ordinal = equivalence_ordinal;
            equivalence_ordinal = equivalence_ordinal.saturating_add(1);
            ordinal
        } else {
            0
        };
        let role = if is_equivalence {
            "anchor"
        } else {
            "declaration"
        };
        let mut node = json!({
            "node_id": id,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": semantic_type,
            "dependencies": dependencies,
            "occurrences": [{"role":role,"ordinal":ordinal}],
            "body": body,
        });
        if let Some(owner) = preimage.get("owner") {
            node["owner"] = owner.clone();
        }
        if let Some(declaration) = declaration {
            node.as_object_mut()
                .expect("node object")
                .insert("declaration".to_owned(), declaration);
        }
        nodes.push(node);
        source_map.push(json!({
            "node_id": id,
            "role": role,
            "ordinal": ordinal,
            "regions": [{"source": source, "start": position, "end": position + 1}],
        }));
    }
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    package["source_map"] = Value::Array(source_map);
    refresh_identity(&mut package);
    package
}

/// Parses an expected RFC 6901 pointer.
pub fn pointer(text: &str) -> JsonPointer {
    JsonPointer::parse(text).unwrap_or_else(|| panic!("not an RFC 6901 pointer: {text:?}"))
}

/// Independently authored expected reader fields. Tests build this value from
/// their chosen input and compare each public field with the real refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedRefusal {
    pub code: CheckedPackageRefusalCode,
    pub path: Option<JsonPointer>,
    pub cause: Option<CheckedPackageRefusalCause>,
    pub locus: Option<CheckedNodeId>,
    pub contract_version: Option<Box<str>>,
    pub document_pointer: Option<JsonPointer>,
}

impl PartialEq<ExpectedRefusal> for CheckedPackageRefusal {
    fn eq(&self, expected: &ExpectedRefusal) -> bool {
        self.code == expected.code
            && self.loci[0].path.as_ref() == expected.path.as_ref()
            && self.cause == expected.cause
            && self.locus == expected.locus
            && self.contract_version == expected.contract_version
            && self.document_pointer == expected.document_pointer
    }
}

impl PartialEq<CheckedPackageRefusal> for ExpectedRefusal {
    fn eq(&self, actual: &CheckedPackageRefusal) -> bool {
        actual == self
    }
}

impl PartialEq<ExpectedRefusal> for CheckedPackageV2ReadResult {
    fn eq(&self, expected: &ExpectedRefusal) -> bool {
        match self {
            Self::Refused(actual) => actual == expected,
            Self::Admitted(_) | Self::Incomplete(_) => false,
        }
    }
}

/// A refusal about the value at the RFC 6901 pointer `path`.
pub fn refusal(code: CheckedPackageRefusalCode, path: &str) -> ExpectedRefusal {
    ExpectedRefusal {
        code,
        path: Some(pointer(path)),
        cause: None,
        locus: None,
        contract_version: None,
        document_pointer: None,
    }
}

/// A refusal at `path` carrying a machine `cause`.
pub fn refusal_cause(
    code: CheckedPackageRefusalCode,
    path: &str,
    cause: CheckedPackageRefusalCause,
) -> ExpectedRefusal {
    ExpectedRefusal {
        cause: Some(cause),
        ..refusal(code, path)
    }
}

/// A refusal about the byte stream rather than a value: it carries no path.
pub fn refusal_bytes(code: CheckedPackageRefusalCode) -> ExpectedRefusal {
    ExpectedRefusal {
        code,
        path: None,
        cause: None,
        locus: None,
        contract_version: None,
        document_pointer: None,
    }
}

/// `unknown_contract_version` at `/contract_version`, carrying the version
/// string the reader read there.
pub fn unknown_version(version: &str) -> ExpectedRefusal {
    ExpectedRefusal {
        code: CheckedPackageRefusalCode::UnknownContractVersion,
        path: Some(pointer("/contract_version")),
        cause: None,
        locus: None,
        contract_version: Some(version.into()),
        document_pointer: None,
    }
}

/// A refusal located at a specific graph node (FR-340 frame refusals):
/// carries the cause tag (absent for a canonical-order defect) and the node
/// key of the offending entry or node.
pub fn refusal_at(
    code: CheckedPackageRefusalCode,
    path: &str,
    cause: Option<CheckedPackageRefusalCause>,
    locus_label: &str,
) -> ExpectedRefusal {
    ExpectedRefusal {
        code,
        path: Some(pointer(path)),
        cause,
        locus: Some(typed_node_id(locus_label)),
        contract_version: None,
        document_pointer: None,
    }
}

/// A refusal located at a complete structural node identity.
pub fn refusal_at_id(
    code: CheckedPackageRefusalCode,
    path: &str,
    cause: Option<CheckedPackageRefusalCause>,
    locus: &Value,
) -> ExpectedRefusal {
    ExpectedRefusal {
        cause,
        locus: Some(serde_json::from_value(locus.clone()).expect("structural locus")),
        ..refusal(code, path)
    }
}

/// The incomplete outcome for `kind`, charged at the value `path` names
/// (`None` only for the byte limit).
pub fn incomplete(
    kind: CheckedPackageLimit,
    limit: u64,
    consumed: u64,
    path: Option<&str>,
) -> CheckedPackageIncomplete {
    CheckedPackageIncomplete {
        limit_kind: kind,
        limit,
        consumed,
        path: path.map(pointer),
    }
}

// ---------------------------------------------------------------------------
// Fixture generator (AGE-1961).
//
// Every document below is *built*, here, from this crate's own public
// vocabulary: `CheckedNodeTag`'s closed families and forms, the
// `quire.application-node/v1` key-derivation formula
// `v2::operations::validate_application_keys` implements (re-derived in
// `application_key` below with the same `sha256_hex`/`canonical` this module
// already used for nominal preimages), and the catalogued operation
// identities `quire-verification-contracts` publishes and this crate's own
// `v2::operations` module validates against (`quire.op.function.call`,
// `quire.op.temporal.clause`, `quire.op.protocol.control`,
// `quire.op.claim.clause` — all four are catalogued with a fixed,
// zero-or-one-law shape this generator matches exactly). No fixture node's
// digest, body or lock entry is read from a file or copied from anywhere
// else; every one is computed by the functions below.

const FIXTURE_SOURCE_AUTHORITY: &str = "agent-ix";
const FIXTURE_SOURCE_IDENTITY: &str = "quire.fixture.source/v1";
const APPLICATION_NODE_VERSION: &str = "quire.application-node/v1";

/// A `CheckedArtifactRef`-shaped definition reference: exactly `{authority,
/// identity}` (QSpec `DefinitionRef`).
pub fn definition_ref(authority: &str, identity: &str) -> Value {
    json!({
        "authority": authority,
        "identity": identity,
    })
}

/// A `CheckedSourceRef`-shaped locked source: exactly `{authority, identity}`.
pub fn source_ref(authority: &str, identity: &str) -> Value {
    json!({
        "authority": authority,
        "identity": identity,
    })
}

/// The `quire.application-node/v1` key this crate's reader
/// (`v2::operations::validate_application_keys`) re-derives for every node
/// whose body is an `application` term: SHA-256 of the RFC-8785 canonical
/// bytes of `{version, node_tag, semantic_form, semantic_type, declaration,
/// recursion, body}`. A protocol clause additionally binds `owner_model`;
/// every undeclared non-clause fixture built here has null declaration and
/// recursion members.
fn application_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &Value,
    body: &Value,
) -> String {
    application_preimage_key(
        node_tag,
        semantic_form,
        semantic_type,
        &Value::Null,
        &Value::Null,
        None,
        body,
    )
}

/// The key QSL FR-092 and FR-094 give an anonymous structural node
/// (`quire.structural-node/v1`: no `declaration`, no `recursion`, no `owner`;
/// `semantic_type` `null` for a node typed by itself). Written here from that
/// preimage, independently of the reader, so a fixture's derived keys are not
/// the reader's own output.
pub fn structural_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: Option<&str>,
    body: &Value,
) -> String {
    let preimage = json!({
        "version": "quire.structural-node/v1",
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": semantic_type.map_or(Value::Null, node_id),
        "declaration": Value::Null,
        "recursion": Value::Null,
        "body": body,
    });
    format!("structural/{preimage}")
}

/// An authored structural path for fixture nodes with an owner or declaration.
fn owned_structural_key(
    node_tag: &str,
    semantic_form: &str,
    _semantic_type: Option<&str>,
    _owner: Value,
    declaration: Value,
) -> String {
    if let Some(names) = declaration["qualified_name"].as_array() {
        return format!(
            "qualified:{}",
            names
                .iter()
                .map(|name| name.as_str().expect("name"))
                .collect::<Vec<_>>()
                .join("/")
        );
    }
    format!("owned/{node_tag}/{semantic_form}")
}

/// A second selected local model declaration in the family package.
pub fn selected_interface_model_node() -> Value {
    let key = owned_structural_key(
        "model",
        "systems_interface",
        None,
        model_owner(FAMILY_MODEL_IDENTITY, FAMILY_INTERFACE),
        Value::Null,
    );
    owned_structural_node(
        plain_node(
            &key,
            "model",
            "systems_interface",
            &key,
            &[],
            empty_aggregate(),
        ),
        model_owner(FAMILY_MODEL_IDENTITY, FAMILY_INTERFACE),
    )
}

/// The all-family fixture with a well-typed protocol finish constraint.
pub fn admitted_protocol_families() -> Value {
    let mut package = v2_all_families();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let clause = nodes
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("protocol clause");
    let finish = nodes
        .iter()
        .position(|node| node["semantic_form"] == "obligation")
        .expect("finish obligation");
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] =
        json!({"term": "reference", "target": over});
    settle(&mut package);
    package
}

/// The derived key of the anonymous `scalar_type`/`boolean` node, as a
/// literal; `structural_key` recomputes it in `tc_226_the_recorded_scalar_keys_are_the_derived_ones`.
pub const BOOLEAN_KEY: &str = "builtin/boolean";

/// The derived key of the anonymous `scalar_type`/`integer` node, as a literal.
pub const INTEGER_KEY: &str = "builtin/integer";

/// The derived key of the anonymous `scalar_type`/`boolean` node.
pub fn boolean_key() -> String {
    BOOLEAN_KEY.to_owned()
}

/// The derived key of the anonymous `scalar_type`/`integer` node.
pub fn integer_key() -> String {
    INTEGER_KEY.to_owned()
}

/// An `aggregate` of one `reference` to `target`: the closed body of a
/// `reference`, `option` and collection node.
pub fn over_body(target: &str) -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "reference", "target": node_id(target)},
    ]})
}

/// An `aggregate` of the `min` then `max` bindings, each an `integer` literal
/// typed at the derived `Integer` key: the closed body of an `integer_range`
/// and a `collection_bounds` node.
pub fn bounds_body(min: &str, max: &str) -> Value {
    let integer = integer_key();
    let bound = |name: &str, value: &str| {
        json!({"term": "binding", "name": name, "value": {
            "term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": value}})
    };
    json!({"term": "aggregate", "members": [bound("min", min), bound("max", max)]})
}

/// The derived key of an anonymous `integer_range` node over `[min, max]`.
pub fn integer_range_key(min: &str, max: &str) -> String {
    structural_key(
        "bounded_domain",
        "integer_range",
        Some(&integer_key()),
        &bounds_body(min, max),
    )
}

/// The `quire.application-node/v1` key of an undeclared application node of
/// `node_tag`/`semantic_form` typed at the node `semantic_type`, over `body`.
pub fn application_node_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &str,
    body: &Value,
) -> String {
    application_key(node_tag, semantic_form, &node_id(semantic_type), body)
}

/// SHA-256 of the RFC-8785 bytes of FR-322's `quire.application-node/v1`
/// preimage, the one builder both fixture keys and grouped keys use.
fn application_preimage_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &Value,
    declaration: &Value,
    recursion: &Value,
    owner_model: Option<&Value>,
    body: &Value,
) -> String {
    let mut preimage = json!({
        "version": APPLICATION_NODE_VERSION,
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": semantic_type,
        "declaration": declaration,
        "recursion": recursion,
        "body": body,
    });
    if let Some(owner_model) = owner_model {
        preimage["owner_model"] = owner_model.clone();
    }
    format!("application/{preimage}")
}
/// QSpec FR-322's `quire.application-node/v1` key for a node inside a
/// recursion group: `recursion` is `{size, ordinal}` of `node` among
/// `group` (member node ids in graph order), and each body `reference` to a
/// group member is keyed as `{term: "group_reference", ordinal}`. Written
/// here from FR-322's text, independently of the reader.
pub fn application_key_in_group(node: &Value, group: &[Value]) -> String {
    fn rewrite(term: &Value, group: &[Value]) -> Value {
        let mut term = term.clone();
        if term["term"] == "reference" {
            if let Some(ordinal) = group.iter().position(|member| *member == term["target"]) {
                return json!({"term": "group_reference", "ordinal": ordinal});
            }
        }
        for key in ["arguments", "members"] {
            if let Some(items) = term.get(key).and_then(Value::as_array).cloned() {
                term[key] = Value::Array(items.iter().map(|item| rewrite(item, group)).collect());
            }
        }
        if term["term"] == "binding" {
            let value = rewrite(&term["value"], group);
            term["value"] = value;
        }
        term
    }
    let ordinal = group
        .iter()
        .position(|member| *member == node["node_id"])
        .expect("node is a group member");
    application_preimage_key(
        node["node_tag"].as_str().expect("tag"),
        node["semantic_form"].as_str().expect("form"),
        &node["semantic_type"],
        &node.get("declaration").cloned().unwrap_or(Value::Null),
        &json!({"size": group.len(), "ordinal": ordinal}),
        node.get("owner_model"),
        &rewrite(&node["body"], group),
    )
}

/// A readable path label for a node in the all-family fixture.
/// These labels are local test identities; the reader resolves the complete
/// `{package, path, ordinal}` value and never re-derives one from node content.
pub fn family_key(prefix: &str) -> String {
    match prefix {
        "aaaa" | "a6a6" => boolean_key(),
        "a3a3" => integer_key(),
        "ffff" => "application/function/pure_function/quire.op.function.call".into(),
        "4040" => "application/temporal/temporal_clause/quire.op.temporal.clause".into(),
        "a2a2" => "application/temporal/formula/quire.op.temporal.eventually".into(),
        "a5a5" => "application/temporal/formula/quire.op.temporal.holds".into(),
        "5050" => "qualified:Example/OrderFlow".into(),
        "p-run" => "qualified:Example/OrderFlow/run/Main".into(),
        "p-finish" => "qualified:Example/OrderFlow/finish/Closed".into(),
        "p-close" => "value/parameter/closed".into(),
        "6060" => "application/claim/verification_claim/quire.op.claim.clause".into(),
        "8080" => "qualified:Example/Function".into(),
        other => format!("fixture/{other}"),
    }
}

fn function_call_body() -> Value {
    json!({
        "term": "application",
        "operator": "call",
        "operation": {
            "identity": "quire.op.function.call",
            "laws": Value::Array(Vec::new()),
            "mode": Value::Null,
            "member": Value::Null,
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [{"term": "reference", "target": node_id(&family_key("8080"))}],
    })
}

/// The lock's own `temporal_profile` selection, reused verbatim as the
/// `laws[]` entry every `temporal.clause`/`claim.clause` application node
/// declares (both are catalogued with that one required role) — the same
/// `CheckedArtifactRef` value in both places, so the operation-law lock join
/// (`is_profile_role` → `lock.profile_selections`) is satisfied by
/// construction rather than by two independently-typed copies. The profile is
/// one of QSpec FR-250's five, a bounded one, which fits the fixture's closed
/// interval `{0, 3}` (FR-038-AC-104, FR-038-AC-108).
fn temporal_profile_law() -> Value {
    json!({
        "role": "temporal_profile",
        "definition": definition_ref(
            FIXTURE_SOURCE_AUTHORITY,
            "quire.temporal.event-position.false-extension/v1",
        ),
    })
}

/// The lock's own `protocol_profile` selection; see [`temporal_profile_law`].
fn protocol_profile_law() -> Value {
    json!({
        "role": "protocol_profile",
        "definition": definition_ref(
            FIXTURE_SOURCE_AUTHORITY,
            "quire.protocol.finite-global/v1",
        ),
    })
}

/// The shared fixture's `quire.op.temporal.clause` body, in the shape the
/// catalog fixes (FR-038-AC-68): six arguments and no member. The first is a
/// `reference` to the `over` `value`/`parameter` node (`a4a4`), the second a
/// `text` literal, the next three empty aggregates, and the sixth a
/// `reference` to the `temporal`/`formula` node (`a2a2`) applying
/// `quire.op.temporal.eventually` with the interval `{0, 3}` over a second
/// formula node (`a5a5`) applying `quire.op.temporal.holds` (FR-038-AC-96).
fn temporal_clause_body() -> Value {
    json!({
        "term": "application",
        "operator": "temporal",
        "operation": {
            "identity": "quire.op.temporal.clause",
            "laws": [temporal_profile_law()],
            "mode": Value::Null,
            "member": Value::Null,
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [
            {"term": "reference", "target": node_id(&family_key("a4a4"))},
            {"term": "literal", "type": node_id(&family_key("a1a1")),
             "value_kind": "text", "value": "clause"},
            empty_aggregate(),
            empty_aggregate(),
            empty_aggregate(),
            {"term": "reference", "target": node_id(&family_key("a2a2"))},
        ],
    })
}

/// The reference targets of [`temporal_clause_body`], digest-ascending: the
/// node's wire `dependencies` (FR-322's application-node join).
fn temporal_clause_dependencies() -> Vec<String> {
    let mut targets = vec![family_key("a4a4"), family_key("a2a2")];
    targets.sort();
    targets
}

/// A `temporal_formula` application of `identity` over `arguments`, with
/// `member`, as the body root of a `temporal`/`formula` node.
fn formula_body(identity: &str, member: Value, arguments: Vec<Value>) -> Value {
    json!({
        "term": "application",
        "operator": "temporal_formula",
        "operation": {
            "identity": identity,
            "laws": Value::Array(Vec::new()),
            "mode": Value::Null,
            "member": member,
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": arguments,
    })
}

/// The fixture's inner formula (`a5a5`): `quire.op.temporal.holds` over a
/// Boolean literal typed at the fixture's own Boolean node (`a6a6`), which
/// no test mutates the way `aaaa`, the family nodes' shared type, is.
fn holds_formula_body() -> Value {
    formula_body(
        "quire.op.temporal.holds",
        Value::Null,
        vec![
            json!({"term": "literal", "type": node_id(&family_key("a6a6")),
                    "value_kind": "boolean", "value": true}),
        ],
    )
}

/// The fixture's root formula (`a2a2`): `quire.op.temporal.eventually` with
/// the interval `{0, 3}` over the `holds` node.
fn eventually_formula_body() -> Value {
    formula_body(
        "quire.op.temporal.eventually",
        json!({"kind": "temporal_interval", "interval": {"lower": "0", "upper": "3"}}),
        vec![json!({"term": "reference", "target": node_id(&family_key("a5a5"))})],
    )
}

fn protocol_control_body() -> Value {
    let over = family_key("a4a4");
    let run = protocol_run_key();
    let finish = protocol_finish_key();
    let reference = |target: &str| json!({"term": "reference", "target": node_id(target)});
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let group = |members: Vec<Value>| json!({"term": "aggregate", "members": members});
    let empty = || group(Vec::new());
    json!({
        "term": "application",
        "operator": "protocol_control",
        "operation": {
            "identity": "quire.op.protocol.control",
            "laws": [protocol_profile_law()],
            "mode": Value::Null,
            "member": {"kind": "profile_operator", "operator": "protocol"},
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [
            binding("over", reference(&over)),
            binding("activation", group(vec![binding("kind", json!({
                "term": "literal", "type": node_id(&family_key("a1a1")),
                "value_kind": "text", "value": "origin"
            }))])),
            binding("captures", empty()),
            binding("roles", empty()),
            binding("relationships", empty()),
            binding("channels", empty()),
            binding("queues", empty()),
            binding("requirements", empty()),
            binding("compensations", empty()),
            binding("terminal", empty()),
            binding("scheduling", empty()),
            binding("run", reference(&run)),
            binding("finish", reference(&finish)),
        ],
    })
}

fn protocol_named_key(_form: &str, declaration: &[&str], _body: &Value) -> String {
    format!("qualified:{}", declaration.join("/"))
}

pub fn protocol_named_node(
    form: &str,
    declaration: &[&str],
    body: Value,
    dependencies: &[String],
) -> Value {
    let key = protocol_named_key(form, declaration, &body);
    let mut ordered = dependencies
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let owner_model = (form == "protocol_clause").then(|| family_key("1515"));
    if let Some(owner_model) = &owner_model {
        ordered.insert(owner_model);
    }
    let mut node = plain_node(
        &key,
        "protocol",
        form,
        &family_key("aaaa"),
        &ordered.into_iter().collect::<Vec<_>>(),
        body,
    );
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["declaration"] = json!({"qualified_name": declaration});
    if let Some(owner_model) = owner_model {
        node["owner_model"] = node_id(&owner_model);
    }
    node
}

/// A declared role target for protocol-reference reader tests.
pub fn protocol_role_node() -> Value {
    let model = family_key("1616");
    protocol_named_node(
        "role",
        &["Example", "OrderFlow", "role", "Worker"],
        json!({
            "term": "application", "operator": "protocol_control",
            "operation": {
                "identity": "quire.op.protocol.control", "laws": [protocol_profile_law()],
                "mode": Value::Null,
                "member": {"kind": "profile_operator", "operator": "role"},
                "leaves": []
            },
            "result_type": node_id(&family_key("aaaa")),
            "arguments": [
                {"term": "binding", "name": "model", "value": {
                    "term": "reference", "target": node_id(&model)
                }},
                {"term": "binding", "name": "replication", "value": {
                    "term": "aggregate", "members": []
                }}
            ]
        }),
        &[model],
    )
}

fn protocol_run_body() -> Value {
    json!({
        "term": "application", "operator": "protocol_control",
        "operation": {
            "identity": "quire.op.protocol.control", "laws": [protocol_profile_law()],
            "mode": Value::Null, "member": {"kind": "profile_operator", "operator": "sequence"},
            "leaves": []
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [{"term": "binding", "name": "children", "value": {
            "term": "aggregate", "members": []
        }}]
    })
}

fn protocol_run_key() -> String {
    protocol_named_key(
        "control",
        &["Example", "OrderFlow", "run", "Main"],
        &protocol_run_body(),
    )
}

fn closing_parameter_body() -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "closed"}},
        {"term": "binding", "name": "level", "value": {
            "term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "1"}}
    ]})
}

fn closing_parameter_key() -> String {
    family_key("p-close")
}

fn protocol_finish_body() -> Value {
    json!({
        "term": "application", "operator": "protocol_control",
        "operation": {
            "identity": "quire.op.protocol.control", "laws": [protocol_profile_law()],
            "mode": Value::Null, "member": {"kind": "profile_operator", "operator": "finish"},
            "leaves": []
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [
            {"term": "binding", "name": "binder", "value": {
                "term": "reference", "target": node_id(&closing_parameter_key())}},
            {"term": "binding", "name": "constraint", "value": {
                "term": "literal", "type": node_id(&family_key("aaaa")),
                "value_kind": "boolean", "value": true}}
        ]
    })
}

fn protocol_finish_key() -> String {
    protocol_named_key(
        "obligation",
        &["Example", "OrderFlow", "finish", "Closed"],
        &protocol_finish_body(),
    )
}

fn claim_clause_body() -> Value {
    json!({
        "term": "application",
        "operator": "claim",
        "operation": {
            "identity": "quire.op.claim.clause",
            "laws": [temporal_profile_law()],
            "mode": Value::Null,
            "member": {"kind": "profile_operator"},
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": Value::Array(Vec::new()),
    })
}

/// A plain graph node: `dependencies` is the node's own wire `dependencies`
/// array (identity-based, distinct from the lowering closure), and
/// `occurrences` is a single `generated`-role occurrence. It carries no
/// `declaration` and no nominal preimage; [`declared`] makes one a source
/// declaration.
fn plain_node(
    digest: &str,
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|d| node_id(d)).collect::<Vec<_>>(),
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    })
}

/// `node` as a source declaration named `qualified_name`: its one
/// occurrence has the `declaration` role and it carries the name.
fn declared(mut node: Value, qualified_name: &[&str]) -> Value {
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["declaration"] = json!({ "qualified_name": qualified_name });
    let source = fixture_source();
    node["owner"] = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
    node
}

/// An empty `aggregate` term: the simplest closed `SemanticTerm` body,
/// charging exactly one unit of `validate_body` work and naming no reference.
fn empty_aggregate() -> Value {
    json!({"term": "aggregate", "members": Value::Array(Vec::new())})
}

/// An application-bodied node with a fixture-authored structural path.
/// `result_type` doubles as the node's own `semantic_type`:
/// an application node's semantic type is the type of the value it produces,
/// the same type its own body already names as `result_type`. Every caller in
/// `build_v2_all_families` happens to pass `"aaaa"` for both, but
/// `build_operation_identities` does not — its call node's real type is its
/// `result_type` argument (`&root`), not the `"aaaa"` family node that
/// document never builds; hardcoding `"aaaa"` here previously left that
/// document's call node with a dangling `semantic_type` that refused
/// admission with `InvalidSemanticGraph` at `semantic_graph.nodes.semantic_type`.
fn application_node(
    node_tag: &str,
    semantic_form: &str,
    operator: &str,
    operation: Value,
    result_type: &str,
    arguments: Vec<Value>,
    dependencies: &[&str],
) -> Value {
    let semantic_type = node_id(result_type);
    let identity = operation["identity"].as_str().expect("operation identity");
    let key = format!("application/{node_tag}/{semantic_form}/{identity}");
    let body = json!({
        "term": "application",
        "operator": operator,
        "operation": operation,
        "result_type": node_id(result_type),
        "arguments": arguments,
    });
    json!({
        "node_id": node_id(&key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": semantic_type,
        "dependencies": dependencies.iter().map(|d| node_id(d)).collect::<Vec<_>>(),
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    })
}

pub fn fixture_source() -> Value {
    source_ref(FIXTURE_SOURCE_AUTHORITY, FIXTURE_SOURCE_IDENTITY)
}

/// A second locked source, distinct from [`fixture_source`]. Nominal
/// packages carry two locked sources so
/// `tc_048_package_id_covers_exactly_the_identity_preimage` can point a
/// source-map entry at the *other* locked source and show that which source
/// a node's occurrences cite is outside the identity preimage; a single
/// locked source could never distinguish that from citing the same source
/// again.
fn fixture_secondary_source() -> Value {
    source_ref(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.source.secondary/v1",
    )
}

/// A locked definition artifact, unreferenced by any node body. Nominal
/// packages carry one so `tc_048_package_id_covers_exactly_the_identity_preimage`
/// can select it into `profile_selections` under a fresh role and show that a
/// preimage member's *value* (not merely its presence) is what the identity
/// covers, reusing an artifact evidence already attests rather than a
/// digest invented for the mutation. It is also load-bearing for a second,
/// unrelated role: `nominal_fixture_members`'s dimension preimage owns it as
/// a `NominalOwner::Definition` (rather than the source-owned declaration and
/// unit), so `tc_048_nominal_cross_field_contradictions_refuse`'s "owner
/// outside lock" case — which mutates `lock.definition_selections[0].identity`
/// — has a real join to break; without this second role that mutation would
/// have nothing in the fixture to affect.
fn fixture_definition_selection() -> Value {
    definition_ref(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.definition.selection/v1",
    )
}

fn fixture_edition_definition() -> Value {
    definition_ref(FIXTURE_SOURCE_AUTHORITY, "quire.fixture.edition/v1")
}

fn fixture_diagnostics_catalog() -> Value {
    definition_ref(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.diagnostics-catalog/v1",
    )
}

fn fixture_capability_report() -> Value {
    json!([{"feature": COMPLETE_VALUE_FEATURE, "disposition": "available"}])
}

fn fixture_lock(profile_selections: Vec<Value>) -> Value {
    json!({
        "sources": [fixture_source()],
        "edition": {"role": "edition", "definition": fixture_edition_definition()},
        "profile_selections": profile_selections,
        "definition_selections": Value::Array(Vec::new()),
        "required_features": [COMPLETE_VALUE_FEATURE],
        "dependency_selections": Value::Array(Vec::new()),
    })
}

/// Rebuilds a fixture package's source map from its nodes' first
/// occurrences, after a test changed a node's occurrences.
pub fn rebuild_source_map(package: &mut Value) {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    let source = package["lock"]["sources"]
        .as_array()
        .and_then(|sources| sources.first())
        .expect("locked fixture source")
        .clone();
    package["source_map"] = source_map_for(&nodes, &source);
}

/// Re-derives the application key of the node at `position` from its own
/// members, after a test changed its body, and renames it everywhere it is
/// referenced.
pub fn rekey_application_node(package: &mut Value, position: usize) {
    let node = package["semantic_graph"]["nodes"][position].clone();
    let fresh = fixture_application_key(&node, &package["semantic_graph"]["nodes"]);
    let stale = node["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    replace_digest(package, &stale, &fresh);
}

/// Collects the targets FR-322's application-node dependency join names in
/// `term`: references and an application's declaration or population link.
/// Returns whether `term` holds an application.
/// Resolve one authored indexed identity against its fixture's own graph table.
pub fn resolved_fixture_id(value: &Value, nodes: &[Value]) -> CheckedNodeId {
    let ids = nodes
        .iter()
        .map(|node| node["node_id"].clone())
        .collect::<Vec<_>>();
    structural_fixture_id_in_graph(value, &ids, &mut Default::default())
}

fn structural_fixture_id_in_graph(
    value: &Value,
    ids: &[Value],
    visiting: &mut std::collections::BTreeSet<usize>,
) -> CheckedNodeId {
    let path = value["path"]
        .as_array()
        .expect("fixture structural path")
        .iter()
        .map(|segment| match segment {
            Value::String(name) => CheckedNodePathSegment::Name(name.as_str().into()),
            Value::Object(object) if object.len() == 1 && object.contains_key("node") => {
                let index = usize::try_from(object["node"].as_u64().expect("fixture node index"))
                    .expect("fixture node index fits usize");
                assert!(visiting.insert(index), "fixture node index cycle");
                let operand = structural_fixture_id_in_graph(
                    ids.get(index).expect("fixture node index in graph"),
                    ids,
                    visiting,
                );
                visiting.remove(&index);
                CheckedNodePathSegment::Node(std::sync::Arc::new(operand))
            }
            Value::Object(_) => CheckedNodePathSegment::Node(std::sync::Arc::new(
                structural_fixture_id_in_graph(segment, ids, visiting),
            )),
            _ => panic!("fixture structural path segment"),
        })
        .collect();
    CheckedNodeId {
        package: value["package"].as_str().expect("fixture package").into(),
        path,
        ordinal: value["ordinal"].as_u64().expect("fixture ordinal"),
    }
}

fn join_targets(
    term: &Value,
    ids: &[Value],
    out: &mut std::collections::BTreeMap<CheckedNodeId, Value>,
) -> bool {
    fn add(out: &mut std::collections::BTreeMap<CheckedNodeId, Value>, id: &Value, ids: &[Value]) {
        let key = structural_fixture_id_in_graph(id, ids, &mut Default::default());
        out.insert(key, id.clone());
    }
    let mut holds_application = false;
    match term["term"].as_str() {
        Some("reference") => {
            add(out, &term["target"], ids);
        }
        Some("application") => {
            holds_application = true;
            if let Some(declaration) = term["operation"]["member"]
                .get("declaration")
                .filter(|value| !value.is_null())
            {
                add(out, declaration, ids);
            }
            if term["operation"]["member"]["kind"] == "fairness" {
                for field in ["declaration", "node", "parallel"] {
                    if let Some(id) = term["operation"]["member"]["target"].get(field) {
                        add(out, id, ids);
                    }
                }
            }
            if matches!(
                term["operation"]["identity"].as_str(),
                Some("quire.op.model.reaches" | "quire.op.model.navigate")
            ) {
                if let Some(population) = term["operation"]["member"]
                    .get("population")
                    .filter(|value| !value.is_null())
                {
                    add(out, population, ids);
                }
            }
            for argument in term["arguments"].as_array().into_iter().flatten() {
                join_targets(argument, ids, out);
            }
        }
        Some("aggregate") => {
            for member in term["members"].as_array().into_iter().flatten() {
                holds_application |= join_targets(member, ids, out);
            }
        }
        Some("binding") => holds_application |= join_targets(&term["value"], ids, out),
        _ => {}
    }
    holds_application
}

/// Gives every node whose body holds an `application` exactly the
/// `dependencies` FR-322's join names in structural node-id order.
fn join_application_dependencies(package: &mut Value) {
    let ids = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(|node| node["node_id"].clone())
        .collect::<Vec<_>>();
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        let mut targets = std::collections::BTreeMap::new();
        if join_targets(&node["body"], &ids, &mut targets) {
            if let Some(owner_model) = node.get("owner_model") {
                let key =
                    structural_fixture_id_in_graph(owner_model, &ids, &mut Default::default());
                targets.insert(key, owner_model.clone());
            }
            node["dependencies"] = Value::Array(targets.into_values().collect());
        }
    }
}

fn fixture_has_application_key(node: &Value) -> bool {
    node["body"]["term"] == "application"
        || (node["node_tag"] == "value" && node["semantic_form"] == "union_value")
}

fn fixture_application_key(node: &Value, nodes: &Value) -> String {
    if let Some(label) = node.get("recursion_group").and_then(Value::as_str) {
        let group = nodes
            .as_array()
            .expect("nodes")
            .iter()
            .filter(|member| member["recursion_group"] == label)
            .map(|member| member["node_id"].clone())
            .collect::<Vec<_>>();
        application_key_in_group(node, &group)
    } else {
        application_preimage_key(
            node["node_tag"].as_str().expect("tag"),
            node["semantic_form"].as_str().expect("form"),
            &node["semantic_type"],
            &node.get("declaration").cloned().unwrap_or(Value::Null),
            &Value::Null,
            node.get("owner_model"),
            &node["body"],
        )
    }
}

/// After a test edited node bodies: re-derives each application-keyed node's
/// dependencies and key, innermost first, renaming each key wherever it is
/// referenced, then the source map and the package identity. A node keyed by
/// something other than its application preimage keeps its key.
pub fn settle(package: &mut Value) {
    join_application_dependencies(package);
    rebuild_source_map(package);
    refresh_identity(package);
}

/// One entry per node, sourced from the node's own single occurrence rather
/// than a hardcoded `generated` role: every node this generator builds
/// carries exactly one entry in its own `occurrences` array, but that entry's
/// role is `declaration` for a declaring node (see `build_operation_identities`'s
/// `declaring_node`) and `generated` for every other one. Reading the role
/// and ordinal back from the node itself, rather than re-asserting
/// `generated` here, is what keeps this function correct for both — a
/// hardcoded `generated` silently produced a `source_map` whose entry
/// disagreed with the declaring node's own recorded occurrence, which
/// `validate_source_map_entries` refuses as `invalid_source_map`.
fn source_map_for(nodes: &[Value], source: &Value) -> Value {
    let entries = nodes
        .iter()
        .enumerate()
        .map(|(position, node)| {
            let occurrence = &node["occurrences"][0];
            json!({
                "node_id": node["node_id"],
                "role": occurrence["role"],
                "ordinal": occurrence["ordinal"],
                "regions": [{"source": source, "start": position, "end": position + 1}],
            })
        })
        .collect::<Vec<_>>();
    Value::Array(entries)
}

/// The base lock/diagnostics/capability skeleton [`nominal_package`] overlays
/// with a caller-supplied node set. Kept separate from [`v2_nominal`] (which
/// *is* one particular overlay of this skeleton — see [`nominal_fixture_members`])
/// so the two do not recurse into each other.
fn nominal_skeleton() -> Value {
    let mut lock = fixture_lock(Vec::new());
    lock["sources"] = json!([fixture_source(), fixture_secondary_source()]);
    lock["definition_selections"] = json!([fixture_definition_selection()]);
    json!({
        "contract_version": "quire.checked-package/v2",
        "name": FIXTURE_PACKAGE,
        "lock": lock,
        "semantic_graph": {
            "graph_version": "quire.checked-semantic-graph/v2",
            "nodes": Value::Array(Vec::new()),
        },
        "source_map": Value::Array(Vec::new()),
        "capability_report": fixture_capability_report(),
        "diagnostics": {
            "catalog": fixture_diagnostics_catalog(),
            "entries": Value::Array(Vec::new()),
        },
    })
}

/// The four nominal preimages [`v2_nominal`] carries, in `(preimage, key)`
/// pairs, in the exact array order every test in this crate that indexes
/// `v2_nominal()`'s nodes by position assumes: `[0] = enum member,
/// [1] = enum declaration, [2] = unit, [3] = dimension`. Digests are computed
/// in dependency order (declaration before its member, dimension before its
/// unit) — required, since each dependant preimage embeds the digest of the
/// node it names — and only then reordered into that display order.
pub fn nominal_fixture_members() -> Vec<(Value, String)> {
    let owner = json!({
        "kind": "source",
        "authority": FIXTURE_SOURCE_AUTHORITY,
        "identity": FIXTURE_SOURCE_IDENTITY,
    });
    let declaration_preimage = json!({
        "version": "quire.enum-declaration-node/v1",
        "owner": owner,
        "qualified_declaration": ["Example", "Status"],
        "ordered": false,
        "members": ["ACTIVE", "DONE"],
    });
    let declaration_key = "qualified:Example/Status".to_owned();

    let member_preimage = json!({
        "version": "quire.enum-member-node/v1",
        "declaration_node_id": node_id(&declaration_key),
        "case": "ACTIVE",
    });
    let member_key = "value/enum_value/ACTIVE".to_owned();

    let dimension_preimage = json!({
        "version": "quire.dimension-node/v1",
        "owner": owner,
        "qualified_declaration": ["Example", "Length"],
        "terms": Value::Array(Vec::new()),
    });
    let dimension_key = "qualified:Example/Length".to_owned();

    let unit_preimage = json!({
        "version": "quire.unit-node/v1",
        "owner": owner,
        "qualified_declaration": ["Example", "Metre"],
        "dimension_node_id": node_id(&dimension_key),
        "target_unit_node_id": Value::Null,
        "scale": {"numerator": "1", "denominator": "1"},
        "offset": {"numerator": "0", "denominator": "1"},
    });
    let unit_key = "qualified:Example/Metre".to_owned();

    vec![
        (member_preimage, member_key),
        (declaration_preimage, declaration_key),
        (unit_preimage, unit_key),
        (dimension_preimage, dimension_key),
    ]
}

/// Builds `v2_all_families()`: one node per V2 semantic-node family (in
/// `CheckedNodeTag::ALL` order, positions 0-12, identified by the historical
/// `aaaa`..`7070` prefixes through [`family_key`]), plus
/// four supporting nodes the family nodes' own required members reference —
/// a second `pure_function` the `function` family's real `application`
/// argument names, the source-declared `object_type` (`Example::Account`,
/// self-typed) and the `process` the `state` family's frame body names, and
/// the dedicated `state`/`frame` node itself, typed by that object type and
/// modifying its `balance` field and the relationship node (the `state`
/// family's own representative node is a plain `snapshot`, so its lowering closure
/// stays the uniform single-`aaaa` shape every other plain family node has;
/// `checked_package_v2_frame_bodies.rs` locates the one real frame node by
/// its `(node_tag, semantic_form)` shape, not by position).
///
/// Every other node is typed by node 0 (`aaaa`, a self-typed `scalar_type`/
/// `boolean`) for simplicity; the two nodes whose lowering closure must reach
/// a second node (`eeee`/`7070`, the `expression` and `correspondence`
/// families) also depend on node 3 (`dddd`). The function/temporal/protocol/
/// claim family nodes (`ffff`/`4040`/`5050`/`6060`) are the only nodes with a
/// real `application` body — the four catalogued operations
/// (`quire.op.function.call`/`.temporal.clause`/`.protocol.control`/
/// `.claim.clause`) this generator's own `plain_node` counterparts would
/// otherwise never exercise.
fn build_v2_all_families() -> Value {
    let aaaa = family_key("aaaa");
    let bbbb = family_key("bbbb");
    let cccc = family_key("cccc");
    let dddd = family_key("dddd");
    let eeee = family_key("eeee");
    let f1010 = family_key("1010");
    let f2020 = family_key("2020");
    let f3030 = family_key("3030");
    let f7070 = family_key("7070");
    let f8080 = family_key("8080");
    let f1515 = family_key("1515");
    let f1616 = family_key("1616");
    // FR-340 orders frame modifications by structural declaration node id,
    // then field name. These two model paths place the object before the
    // relationship.
    let mut modifies = [
        (
            &f1515,
            json!({"kind": "field", "declaration": node_id(&f1515), "name": "balance", "scope": "every-object"}),
        ),
        (
            &f2020,
            json!({"kind": "relationship", "declaration": node_id(&f2020)}),
        ),
    ];
    modifies.sort_by(|left, right| left.0.cmp(right.0));
    let frame_body = json!({
        "term": "frame",
        "modifies": modifies.into_iter().map(|(_, entry)| entry).collect::<Vec<_>>(),
        "creates": [node_id(&f1515)],
        "deletes": [node_id(&f1616)],
        "deletes_scoped": [],
    });
    let frame_key = "state/frame".to_owned();

    let nodes = vec![
        plain_node(
            &aaaa,
            "scalar_type",
            "boolean",
            &aaaa,
            &[],
            empty_aggregate(),
        ),
        // No wire `dependencies` of its own: its lowering closure already
        // reaches `aaaa` through `semantic_type` alone, and leaving this one
        // family node's `dependencies` empty in the base document is what lets
        // `complete_v1_checked_package.rs`'s own edges accounting test prove
        // the exact/one-over boundary by adding exactly one dependency to it.
        plain_node(
            &bbbb,
            "composite_type",
            "option",
            &bbbb,
            &[],
            over_body(&aaaa),
        ),
        plain_node(
            &cccc,
            "bounded_domain",
            "integer_range",
            &family_key("a3a3"),
            &[family_key("a3a3").as_str()],
            bounds_body("0", "1000"),
        ),
        plain_node(
            &dddd,
            "value",
            "literal",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &eeee,
            "expression",
            "reference",
            &aaaa,
            &[aaaa.as_str(), dddd.as_str()],
            json!({"term": "reference", "target": node_id(&dddd)}),
        ),
        application_node(
            "function",
            "pure_function",
            "call",
            function_call_body()["operation"].clone(),
            &aaaa,
            vec![json!({"term": "reference", "target": node_id(&f8080)})],
            // FR-322's application-node join: exactly the body's reference
            // targets.
            &[f8080.as_str()],
        ),
        plain_node(
            &f1010,
            "model",
            "namespace",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        {
            let mut relation = plain_node(
                &f2020,
                "relation",
                "relationship",
                &f2020,
                &[aaaa.as_str()],
                empty_aggregate(),
            );
            relation["owner"] = model_owner(FAMILY_MODEL_IDENTITY, FAMILY_RELATIONSHIP);
            relation
        },
        plain_node(
            &f3030,
            "state",
            "snapshot",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        application_node(
            "temporal",
            "temporal_clause",
            "temporal",
            temporal_clause_body()["operation"].clone(),
            &aaaa,
            temporal_clause_body()["arguments"]
                .as_array()
                .expect("arguments")
                .clone(),
            &temporal_clause_dependencies()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        protocol_named_node(
            "protocol_clause",
            &["Example", "OrderFlow"],
            protocol_control_body(),
            &[
                family_key("a4a4"),
                protocol_run_key(),
                protocol_finish_key(),
            ],
        ),
        application_node(
            "claim",
            "verification_claim",
            "claim",
            claim_clause_body()["operation"].clone(),
            &aaaa,
            Vec::new(),
            &[],
        ),
        plain_node(
            &f7070,
            "correspondence",
            "source_locus",
            &aaaa,
            &[aaaa.as_str(), dddd.as_str()],
            empty_aggregate(),
        ),
        declared(
            plain_node(
                &f8080,
                "function",
                "pure_function",
                &aaaa,
                &[],
                empty_aggregate(),
            ),
            &["Example", "Function"],
        ),
        owned_structural_node(
            plain_node(
                &f1515,
                "model",
                "object_type",
                &f1515,
                &[aaaa.as_str()],
                empty_aggregate(),
            ),
            model_owner(FAMILY_MODEL_IDENTITY, FAMILY_OBJECT),
        ),
        owned_structural_node(
            plain_node(&f1616, "model", "process", &f1616, &[], empty_aggregate()),
            model_owner(FAMILY_MODEL_IDENTITY, FAMILY_PROCESS),
        ),
        plain_node(
            &frame_key,
            "state",
            "frame",
            &f1515,
            &[f1515.as_str(), f1616.as_str(), f2020.as_str()],
            frame_body,
        ),
        // The nodes the temporal clause's arguments name: the `text` type of
        // its name literal, the `temporal`/`formula` node it applies
        // (`eventually` over `holds`) and its `over` parameter.
        plain_node(
            &family_key("a1a1"),
            "scalar_type",
            "text",
            &family_key("a1a1"),
            &[],
            empty_aggregate(),
        ),
        application_node(
            "temporal",
            "formula",
            "temporal_formula",
            eventually_formula_body()["operation"].clone(),
            &aaaa,
            eventually_formula_body()["arguments"]
                .as_array()
                .expect("arguments")
                .clone(),
            &[family_key("a5a5").as_str()],
        ),
        application_node(
            "temporal",
            "formula",
            "temporal_formula",
            holds_formula_body()["operation"].clone(),
            &aaaa,
            holds_formula_body()["arguments"]
                .as_array()
                .expect("arguments")
                .clone(),
            // A literal's `type` is no dependency (FR-322).
            &[],
        ),
        plain_node(
            &family_key("a3a3"),
            "scalar_type",
            "integer",
            &family_key("a3a3"),
            &[],
            empty_aggregate(),
        ),
        // The clause's `over` parameter: QSL FR-092's `{name, level}` body,
        // typed at the Boolean node, carrying the `expression` role.
        {
            let mut parameter = plain_node(
                &family_key("a4a4"),
                "value",
                "parameter",
                &aaaa,
                &[],
                json!({
                    "term": "aggregate",
                    "members": [
                        {"term": "binding", "name": "name", "value": {
                            "term": "literal", "type": node_id(&family_key("a1a1")),
                            "value_kind": "text", "value": "over"}},
                        {"term": "binding", "name": "level", "value": {
                            "term": "literal", "type": node_id(&family_key("a3a3")),
                            "value_kind": "integer", "value": "0"}},
                    ],
                }),
            );
            parameter["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
            parameter
        },
        protocol_named_node(
            "control",
            &["Example", "OrderFlow", "run", "Main"],
            protocol_run_body(),
            &[],
        ),
        protocol_named_node(
            "obligation",
            &["Example", "OrderFlow", "finish", "Closed"],
            protocol_finish_body(),
            &[closing_parameter_key()],
        ),
        {
            let mut parameter = plain_node(
                &closing_parameter_key(),
                "value",
                "parameter",
                &aaaa,
                &[],
                closing_parameter_body(),
            );
            parameter["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
            parameter
        },
    ];

    let source = fixture_source();
    let profile_selections = vec![temporal_profile_law(), protocol_profile_law()];
    let mut package = json!({
        "contract_version": "quire.checked-package/v2",
        "name": FIXTURE_PACKAGE,
        "lock": fixture_lock(profile_selections),
        "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
        "source_map": Value::Array(Vec::new()),
        "capability_report": fixture_capability_report(),
        "diagnostics": {"catalog": fixture_diagnostics_catalog(), "entries": Value::Array(Vec::new())},
    });
    let nodes_array = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    package["source_map"] = source_map_for(&nodes_array, &source);
    select_model(&mut package, FAMILY_MODEL_IDENTITY);
    settle(&mut package);
    package
}

/// Builds `positive_operation_identities()`: a small five-node document
/// carrying one node of each shape
/// `tc_048_deleting_a_declared_wire_member_refuses_before_the_projection_compare`
/// needs to delete a member from — a `declaration`, a `literal.type`, an
/// `application.operation` and an `application.result_type` — so it can
/// assert the closed-schema refusal each deletion produces.
fn build_operation_identities() -> Value {
    let root = boolean_key();
    let source = fixture_source();
    let declaring = owned_structural_key(
        "model",
        "namespace",
        Some(&root),
        source_owner(&source),
        json!({"qualified_name": ["Example", "Widget"]}),
    );
    let literal_body =
        json!({"term": "literal", "type": node_id(&root), "value_kind": "boolean", "value": true});
    let literal_key = structural_key("value", "literal", Some(&root), &literal_body);
    let func = owned_structural_key(
        "function",
        "pure_function",
        Some(&root),
        source_owner(&source),
        json!({"qualified_name": ["Example", "Function"]}),
    );

    let root_node = plain_node(
        &root,
        "scalar_type",
        "boolean",
        &root,
        &[],
        empty_aggregate(),
    );

    let declaring_node = declared(
        plain_node(
            &declaring,
            "model",
            "namespace",
            &root,
            &[root.as_str()],
            empty_aggregate(),
        ),
        &["Example", "Widget"],
    );

    let literal_node = plain_node(
        &literal_key,
        "value",
        "literal",
        &root,
        &[root.as_str()],
        literal_body,
    );

    let func_node = declared(
        plain_node(
            &func,
            "function",
            "pure_function",
            &root,
            &[],
            empty_aggregate(),
        ),
        &["Example", "Function"],
    );

    let call_node = application_node(
        "expression",
        "call",
        "call",
        json!({
            "identity": "quire.op.function.call", "laws": Value::Array(Vec::new()),
            "mode": Value::Null, "member": Value::Null, "leaves": Value::Array(Vec::new()),
        }),
        &root,
        vec![json!({"term": "reference", "target": node_id(&func)})],
        &[func.as_str()],
    );

    let nodes = vec![
        root_node,
        declaring_node,
        literal_node,
        func_node,
        call_node,
    ];
    let source = fixture_source();
    let mut package = json!({
        "contract_version": "quire.checked-package/v2",
        "name": FIXTURE_PACKAGE,
        "lock": fixture_lock(Vec::new()),
        "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
        "source_map": Value::Array(Vec::new()),
        "capability_report": fixture_capability_report(),
        "diagnostics": {"catalog": fixture_diagnostics_catalog(), "entries": Value::Array(Vec::new())},
    });
    let nodes_array = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    package["source_map"] = source_map_for(&nodes_array, &source);
    refresh_identity(&mut package);
    package
}
