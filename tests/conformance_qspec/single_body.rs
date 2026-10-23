// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published single-body wire vectors, read from the owning QSpec checkout.

use super::{canonical_digest, fixtures_dir, read_json, read_package, POSITIVE_ALL};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    encode_checked_package_v2, CheckedPackageIdentityPreimageRefV2, CheckedPackageLockV2,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedSemanticGraphV2,
};
use serde_json::{json, Value};

fn fixture(name: &str) -> Value {
    read_json(&fixtures_dir().join(name)).unwrap_or_else(|why| panic!("{why}"))
}

/// Independent QSpec derivation: preserve every node member except occurrences.
fn projection(package: &Value) -> Value {
    Value::Array(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("published graph nodes")
            .iter()
            .map(|node| {
                let mut node = node.as_object().expect("published node object").clone();
                assert!(node.remove("occurrences").is_some(), "node occurrences");
                Value::Object(node)
            })
            .collect(),
    )
}

fn hash_input(package: &Value) -> Value {
    let mut preimage = package["identity_preimage"]
        .as_object()
        .expect("published selection object")
        .clone();
    assert!(!preimage.contains_key("identity_projection"));
    preimage.insert("identity_projection".into(), projection(package));
    Value::Object(preimage)
}

fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    match read_package(package).expect("published evidence can be prepared") {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("single-body package must admit, got {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn published_single_body_packages_keep_their_recorded_identity_and_writer_bytes() {
    for name in POSITIVE_ALL {
        let package = fixture(name);
        assert_eq!(
            canonical_digest(&hash_input(&package)),
            package["package_id"]["digest"]
                .as_str()
                .expect("recorded digest"),
            "{name}: independent graph-derived identity"
        );
        let read = admitted(&package);
        assert_eq!(
            serde_json::to_value(read.package_id()).unwrap(),
            package["package_id"]
        );
        let selection = read.identity_preimage();
        let preimage = CheckedPackageIdentityPreimageRefV2 {
            version: &selection.version,
            edition: &selection.edition,
            profile_selections: &selection.profile_selections,
            definition_selections: &selection.definition_selections,
            model_selections: &selection.model_selections,
            required_features: &selection.required_features,
            dependency_selections: &selection.dependency_selections,
            identity_projection: &read.graph().nodes,
        };
        assert_eq!(
            quire_canonical::sha256(&preimage, Limits::new(1 << 24))
                .unwrap()
                .to_string(),
            package["package_id"]["digest"],
            "{name}: production borrowed hash input"
        );
        let bytes = encode_checked_package_v2(
            &preimage,
            read.package_id(),
            read.lock(),
            read.graph(),
            read.source_map(),
            read.capability_report(),
            read.diagnostics(),
            Limits::new(1 << 24),
        )
        .expect("published envelope encodes");
        let encoded: Value = serde_json::from_slice(&bytes).expect("encoded JSON");
        assert_eq!(
            encoded, package,
            "{name}: no second wire body or lost member"
        );
        assert_eq!(
            bytes,
            quire_canonical::to_vec(&package, Limits::new(1 << 24)).unwrap(),
            "{name}: complete canonical writer bytes"
        );
        assert_eq!(admitted(&encoded).package_id(), read.package_id());
    }
}

// These two tests bind the published external requirement directly. The IR
// consumer-spec mapping is part of the coordinated spec amendment, not inferred
// from a retired projection-owner criterion.
fn shipped_projection_vector_refuses_at_the_first_wire_member(selected: &str) {
    let schema = read_json(&fixtures_dir().join("../schema.json")).expect("published schema");
    let selection_schema = json!({
        "$ref": "#/$defs/IdentityPreimage",
        "$defs": schema["$defs"],
    });
    let selection_schema = jsonschema::JSONSchema::compile(&selection_schema)
        .expect("published selection schema compiles");
    let base = fixture("positive-all-families.json");
    let base_id = base["package_id"].clone();
    assert!(selection_schema.is_valid(&base["identity_preimage"]));
    let adverse = fixture("adverse.json");
    let rows = adverse["shipped_projection_cases"]
        .as_array()
        .expect("published cases");
    assert!(
        !rows.is_empty(),
        "the published negative vector list must not disappear"
    );
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let id = row["id"].as_str().expect("vector id");
        if id != selected {
            continue;
        }
        assert!(seen.insert(id), "duplicate vector {id}");
        assert_eq!(row["base"], "positive-all-families");
        let value = match &row["identity_projection"] {
            Value::String(kind) if kind == "occurrence-free-graph" => projection(&base),
            Value::Array(values) if values.is_empty() => json!([]),
            other => panic!("{id}: unhandled published projection recipe {other:?}"),
        };
        let mut candidate = base.clone();
        candidate["identity_preimage"]["identity_projection"] = value;
        assert!(
            !selection_schema.is_valid(&candidate["identity_preimage"]),
            "{id}"
        );
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&candidate).expect("published evidence")
        else {
            panic!("{id}: old shipped projection must refuse before identity validation");
        };
        assert_eq!(row["expected_code"], "malformed_wire");
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MalformedWire,
            "{id}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            row["expected_locus"].as_str(),
            "{id}: first refusal follows the published locus"
        );
        assert_eq!(refusal.cause, None, "{id}: no later identity mismatch");
        candidate["identity_preimage"]
            .as_object_mut()
            .unwrap()
            .remove("identity_projection");
        assert_eq!(candidate, base, "{id}: exact restoration");
        assert_eq!(
            serde_json::to_value(admitted(&candidate).package_id()).unwrap(),
            base_id,
            "{id}"
        );
    }
    assert!(
        seen.contains(selected),
        "selected published vector is present"
    );
}

#[trace("TC-048", "FR-038-AC-4")]
#[test]
fn derived_identity_binds_a_nominal_body_and_node_order_but_excludes_occurrences() {
    let base = fixture("positive-nominal-identities.json");
    let original_id = admitted(&base).package_id().clone();
    let original_hash = canonical_digest(&hash_input(&base));
    assert_eq!(original_hash, original_id.digest.as_ref());
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| {
            node.get("nominal_identity_preimage").is_some() && node["body"]["term"] == "literal"
        })
        .expect("published nominal literal");
    let mut changed = base.clone();
    changed["semantic_graph"]["nodes"][position]["body"]["value"] = json!("ALTERED");
    assert_eq!(
        changed["semantic_graph"]["nodes"][position]["node_id"],
        base["semantic_graph"]["nodes"][position]["node_id"],
        "the nominal key alone cannot substitute for its full body in package identity"
    );
    let changed_hash = canonical_digest(&hash_input(&changed));
    assert_ne!(changed_hash, original_hash);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&changed).unwrap() else {
        panic!("the original package id must not authenticate a changed nominal body");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::StaleDependency);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::ContentMismatch)
    );
    assert_eq!(refusal.path.unwrap().to_string(), "/package_id/digest");
    assert_eq!(refusal.actual.unwrap().digest.as_ref(), changed_hash);

    let mut reordered = base.clone();
    reordered["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_ne!(canonical_digest(&hash_input(&reordered)), original_hash);
    reordered["package_id"]["digest"] = json!(canonical_digest(&hash_input(&reordered)));
    admitted(&reordered);

    let mut relocated = base;
    for node in relocated["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        node["occurrences"] = json!([]);
    }
    assert_eq!(canonical_digest(&hash_input(&relocated)), original_hash);
    // Deliberately do not claim admission: occurrence/source-map consistency is
    // an independent obligation, and this only tests the identity exclusion.
}

/// Hash the graph through the production borrowed encoder before admission.
/// This tests the producer API without bypassing the reader trust boundary.
fn producer_digest(package: &Value) -> String {
    let lock: CheckedPackageLockV2 = serde_json::from_value(package["lock"].clone()).unwrap();
    let graph: CheckedSemanticGraphV2 =
        serde_json::from_value(package["semantic_graph"].clone()).unwrap();
    let input = CheckedPackageIdentityPreimageRefV2 {
        version: package["identity_preimage"]["version"].as_str().unwrap(),
        edition: &lock.edition,
        profile_selections: &lock.profile_selections,
        definition_selections: &lock.definition_selections,
        model_selections: &lock.model_selections,
        required_features: &lock.required_features,
        dependency_selections: &lock.dependency_selections,
        identity_projection: &graph.nodes,
    };
    quire_canonical::sha256(&input, Limits::new(1 << 24))
        .unwrap()
        .to_string()
}

#[trace("TC-048", "FR-038-AC-4", "FR-038-AC-310")]
#[test]
fn published_selection_schema_and_producer_identity_agree_for_every_fixture() {
    let schema = read_json(&fixtures_dir().join("../schema.json")).unwrap();
    // This repository's validator supports the selection fragment. The full
    // owning schema uses Draft 2020-12 (including prefixItems); it is checked
    // separately with a validator for that draft, not reinterpreted as Draft 7.
    let selection_schema = json!({
        "$ref": "#/$defs/IdentityPreimage",
        "$defs": schema["$defs"],
    });
    let validator = jsonschema::JSONSchema::compile(&selection_schema).unwrap();
    let mut names: Vec<_> = std::fs::read_dir(fixtures_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.starts_with("positive-") && name.ends_with(".json"))
        .collect();
    names.sort();
    assert!(names.len() >= POSITIVE_ALL.len());
    for name in names {
        let package = fixture(&name);
        assert!(
            validator.is_valid(&package["identity_preimage"]),
            "{name}: published selection schema"
        );
        assert_eq!(package["identity_preimage"].as_object().unwrap().len(), 7);
        assert_eq!(
            canonical_digest(&hash_input(&package)),
            package["package_id"]["digest"],
            "{name}: normative occurrence-free graph preimage"
        );
        assert_eq!(
            producer_digest(&package),
            package["package_id"]["digest"],
            "{name}: production borrowed hash encoder"
        );
    }
}

#[trace("TC-048", "FR-038-AC-4")]
#[test]
fn producer_identity_covers_full_nominal_body_and_order_excluding_occurrences() {
    let base = fixture("positive-nominal-identities.json");
    let original = producer_digest(&base);
    assert_eq!(original, base["package_id"]["digest"]);
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| {
            node.get("nominal_identity_preimage").is_some() && node["body"]["term"] == "literal"
        })
        .unwrap();
    let mut changed = base.clone();
    changed["semantic_graph"]["nodes"][position]["body"]["value"] = json!("ALTERED");
    assert_eq!(
        changed["semantic_graph"]["nodes"][position]["node_id"],
        base["semantic_graph"]["nodes"][position]["node_id"]
    );
    assert_ne!(producer_digest(&changed), original);
    assert_eq!(
        producer_digest(&changed),
        canonical_digest(&hash_input(&changed))
    );
    let mut reordered = base.clone();
    reordered["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_ne!(producer_digest(&reordered), original);
    assert_eq!(
        producer_digest(&reordered),
        canonical_digest(&hash_input(&reordered))
    );
    let mut relocated = base;
    for node in relocated["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        node["occurrences"] = json!([]);
    }
    assert_eq!(producer_digest(&relocated), original);
}

#[trace("TC-048", "FR-038-AC-310")]
#[test]
fn published_shipped_derived_projection_refuses_at_the_first_wire_member() {
    shipped_projection_vector_refuses_at_the_first_wire_member(
        "identity-preimage-ships-derived-projection",
    );
}

#[trace("TC-048", "FR-038-AC-310")]
#[test]
fn published_shipped_empty_projection_refuses_at_the_first_wire_member() {
    shipped_projection_vector_refuses_at_the_first_wire_member(
        "identity-preimage-ships-empty-projection",
    );
}

/// The selection object is closed for every immediate member, not just the
/// retired projection; nested selection records retain their own refusal rules.
#[trace("TC-048", "FR-038-AC-310")]
#[test]
fn unknown_wire_selection_member_refuses_at_that_member() {
    for member in [
        "future_selection",
        "",
        "future/selection",
        "future~selection",
    ] {
        let mut package = fixture("positive-all-families.json");
        package["identity_preimage"][member] = json!([]);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&package).unwrap() else {
            panic!("the seven-member wire selection must stay closed");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MalformedWire,
            "{member:?}"
        );
        let expected = format!(
            "/identity_preimage/{}",
            member.replace('~', "~0").replace('/', "~1")
        );
        assert_eq!(refusal.path.unwrap().as_str(), expected, "{member:?}");
    }
}
