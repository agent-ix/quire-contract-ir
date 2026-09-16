// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! CheckedPackage V1 to V2 migration correspondence, driven by the vendored
//! QSpec migration vectors.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{apply_patch, canonical, evidence_for, fixture};
use ix_trace_rs::trace;
use quire_contract_ir::{
    migrate_checked_package, read_checked_package, CheckedPackage, CheckedPackageDispatchResult,
    CheckedPackageMigrationRequest, CheckedPackageReadLimits, CheckedPackageV2, MigrationOutcome,
};
use serde_json::{json, Value};

const VECTOR_ROOT: &str = "checked-package-v2";

fn read_v1(path: &str) -> CheckedPackage {
    let value = fixture(&format!("{VECTOR_ROOT}/{path}"));
    match read_checked_package(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
    ) {
        CheckedPackageDispatchResult::AdmittedV1(package) => *package,
        other => panic!("migration source {path} must admit as V1, got {other:?}"),
    }
}

fn read_v2(path: &str) -> CheckedPackageV2 {
    let value = fixture(&format!("{VECTOR_ROOT}/{path}"));
    match read_checked_package(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("migration target {path} must admit as V2, got {other:?}"),
    }
}

/// The request a caller supplies: the claimed target and reconstruction inputs.
fn request(correspondence: &Value) -> CheckedPackageMigrationRequest {
    let mut members = serde_json::Map::new();
    members.insert(
        "target_package_id".into(),
        correspondence["target_package_id"].clone(),
    );
    if let Some(inputs) = correspondence.get("reconstruction_inputs") {
        members.insert("reconstruction_inputs".into(), inputs.clone());
    }
    serde_json::from_value(Value::Object(members)).expect("migration request")
}

fn outcome(source: &CheckedPackage, target: &CheckedPackageV2, correspondence: &Value) -> Value {
    let result: MigrationOutcome =
        migrate_checked_package(source, target, &request(correspondence));
    serde_json::to_value(result).expect("migration outcome JSON")
}

struct Positive {
    source: CheckedPackage,
    target: CheckedPackageV2,
    outcome: Value,
}

fn positives() -> Vec<(String, Positive)> {
    let vectors = fixture(&format!(
        "{VECTOR_ROOT}/migration-correspondence-vectors.json"
    ));
    vectors["positives"]
        .as_array()
        .expect("positives")
        .iter()
        .map(|vector| {
            (
                vector["name"].as_str().expect("name").to_owned(),
                Positive {
                    source: read_v1(vector["source_package"].as_str().expect("source")),
                    target: read_v2(vector["target_package"].as_str().expect("target")),
                    outcome: vector["outcome"].clone(),
                },
            )
        })
        .collect()
}

/// Tracing: TC-049, FR-038-AC-6
#[trace("TC-049", "FR-038-AC-6")]
#[test]
fn tc_049_migration_reproduces_every_vendored_positive_and_refusal() {
    let schema = fixture(&format!(
        "{VECTOR_ROOT}/migration-correspondence.schema.json"
    ));
    let compiled = jsonschema::JSONSchema::compile(&schema).expect("vendored schema compiles");
    let positives = positives();
    assert_eq!(positives.len(), 2);
    for (name, positive) in &positives {
        let correspondence = &positive.outcome["correspondence"];
        let actual = outcome(&positive.source, &positive.target, correspondence);
        assert_eq!(actual, positive.outcome, "{name}");
        assert!(
            compiled.is_valid(&actual["correspondence"]),
            "{name} schema"
        );
    }
    let rekeyed = &positives[1].1.outcome["correspondence"]["node_correspondences"];
    assert!(rekeyed
        .as_array()
        .expect("node correspondences")
        .iter()
        .any(|node| node["basis"] == json!("relinked-rekeyed")));

    let vectors = fixture(&format!(
        "{VECTOR_ROOT}/migration-correspondence-vectors.json"
    ));
    let refusals = vectors["refusals"].as_array().expect("refusals");
    assert_eq!(refusals.len(), 17);
    for refusal in refusals {
        let name = refusal["name"].as_str().expect("name");
        let (_, base) = positives
            .iter()
            .find(|(candidate, _)| refusal["base"] == json!(candidate))
            .expect("refusal base vector");
        let mut correspondence = base.outcome["correspondence"].clone();
        apply_patch(&mut correspondence, &refusal["patch"]);
        assert_eq!(
            outcome(&base.source, &base.target, &correspondence),
            refusal["expected_outcome"],
            "{name}"
        );
    }
}

/// Tracing: TC-049, FR-038-AC-6
#[trace("TC-049", "FR-038-AC-6")]
#[test]
fn tc_049_migration_refuses_mismatched_targets_and_orders_refusals() {
    let positives = positives();
    let (all, nominal) = (&positives[0].1, &positives[1].1);
    let all_request = &all.outcome["correspondence"];

    // A correctly identified but different V2 package is not a relink target.
    let mut mismatched = all_request.clone();
    mismatched["target_package_id"] = json!(nominal.target.package_id());
    assert_eq!(
        outcome(&all.source, &nominal.target, &mismatched),
        json!({"outcome":"refused","code":"migration_target_incompatible","subject":{"input":"target_package_id"}})
    );
    // The V1 identity relabelled as V2 does not name the target.
    let mut relabelled = all_request.clone();
    relabelled["target_package_id"] = json!(all.source.package_id());
    assert_eq!(
        outcome(&all.source, &all.target, &relabelled),
        json!({"outcome":"refused","code":"migration_target_incompatible","subject":{"input":"target_package_id"}})
    );

    // Same lock and node count, but one ordinal changes family.
    let mut family_value = fixture(&format!(
        "{VECTOR_ROOT}/fixtures/positive-all-families.json"
    ));
    family_value["semantic_graph"]["nodes"][1]["node_tag"] = json!("scalar_type");
    family_value["semantic_graph"]["nodes"][1]["semantic_form"] = json!("boolean");
    checked_package::refresh_identity(&mut family_value);
    let family_target = match read_checked_package(
        &canonical(&family_value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&family_value),
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        other => panic!("family-changed target must admit, got {other:?}"),
    };
    let mut family = all_request.clone();
    family["target_package_id"] = json!(family_target.package_id());
    assert_eq!(
        outcome(&all.source, &family_target, &family),
        json!({"outcome":"refused","code":"migration_target_incompatible","subject":{"input":"target_package_id"}})
    );

    // byte_only precedes missing, missing precedes stale, stale precedes target.
    let mut byte_only = all_request.clone();
    byte_only["target_package_id"]["digest"] = json!("0".repeat(64));
    byte_only
        .as_object_mut()
        .expect("correspondence")
        .remove("reconstruction_inputs");
    assert_eq!(
        outcome(&all.source, &all.target, &byte_only)["code"],
        json!("migration_byte_only_reconstruction")
    );
    let mut missing_over_stale = all_request.clone();
    missing_over_stale["reconstruction_inputs"]["sources"][0]["digest"] = json!("3".repeat(64));
    missing_over_stale["reconstruction_inputs"]
        .as_object_mut()
        .expect("inputs")
        .remove("dependency_selections");
    assert_eq!(
        outcome(&all.source, &all.target, &missing_over_stale),
        json!({"outcome":"refused","code":"migration_input_missing","subject":{"input":"dependency_selections"}})
    );
    let mut stale_over_target = all_request.clone();
    stale_over_target["reconstruction_inputs"]["edition"]["definition"]["digest"] =
        json!("7".repeat(64));
    stale_over_target["target_package_id"]["digest"] = json!("0".repeat(64));
    assert_eq!(
        outcome(&all.source, &all.target, &stale_over_target),
        json!({"outcome":"refused","code":"migration_input_stale","subject":{"input":"edition"}})
    );
    let mut ambiguous_over_stale = nominal.outcome["correspondence"].clone();
    let sources = ambiguous_over_stale["reconstruction_inputs"]["sources"]
        .as_array_mut()
        .expect("sources");
    let duplicate = sources[1].clone();
    sources.push(duplicate);
    ambiguous_over_stale["reconstruction_inputs"]["edition"]["definition"]["digest"] =
        json!("7".repeat(64));
    assert_eq!(
        outcome(&nominal.source, &nominal.target, &ambiguous_over_stale),
        json!({"outcome":"refused","code":"migration_input_ambiguous","subject":{"input":"sources","authority":"agent-ix","identity":"example-units"}})
    );

    // Every artifact-list input refuses a repeated artifact as ambiguous, after
    // missing and before stale.
    let nominal_request = &nominal.outcome["correspondence"];
    let definition = nominal_request["reconstruction_inputs"]["definition_selections"][0].clone();
    let model = json!({
        "authority":"agent-ix","identity":"example-compiled","revision":{"namespace":"git","value":"1"},
        "digest_domain":"quire.compiled-model.bytes/v1","digest":"8".repeat(64),"export":"Example"
    });
    let ambiguous_cases = [
        (
            "definition_selections",
            json!([definition.clone(), definition.clone()]),
            "example-model",
        ),
        (
            "model_selections",
            json!([model.clone(), model.clone()]),
            "example-compiled",
        ),
    ];
    for (input, duplicated, identity) in ambiguous_cases {
        let mut ambiguous = nominal_request.clone();
        ambiguous["reconstruction_inputs"][input] = duplicated.clone();
        ambiguous["reconstruction_inputs"]["edition"]["definition"]["digest"] =
            json!("7".repeat(64));
        assert_eq!(
            outcome(&nominal.source, &nominal.target, &ambiguous),
            json!({"outcome":"refused","code":"migration_input_ambiguous","subject":{"input":input,"authority":"agent-ix","identity":identity}}),
            "{input} ambiguous over stale"
        );
        let mut missing = nominal_request.clone();
        missing["reconstruction_inputs"][input] = duplicated;
        missing["reconstruction_inputs"]
            .as_object_mut()
            .expect("inputs")
            .remove("dependency_selections");
        assert_eq!(
            outcome(&nominal.source, &nominal.target, &missing),
            json!({"outcome":"refused","code":"migration_input_missing","subject":{"input":"dependency_selections"}}),
            "{input} missing over ambiguous"
        );
    }
    // Two exports of one compiled model are distinct artifacts, not ambiguous;
    // they are refused only as stale against a lock that selects neither.
    let mut exports = nominal_request.clone();
    let mut other_export = model.clone();
    other_export["export"] = json!("Other");
    exports["reconstruction_inputs"]["model_selections"] = json!([model, other_export]);
    assert_eq!(
        outcome(&nominal.source, &nominal.target, &exports),
        json!({"outcome":"refused","code":"migration_input_stale","subject":{"input":"model_selections"}})
    );

    // Sources are a set in both the stale and the target check: reordering
    // them still relinks, and the receipt records the inputs as supplied.
    let mut reordered = nominal_request.clone();
    reordered["reconstruction_inputs"]["sources"]
        .as_array_mut()
        .expect("sources")
        .reverse();
    let relinked = outcome(&nominal.source, &nominal.target, &reordered);
    assert_eq!(relinked["outcome"], json!("relinked"));
    assert_eq!(
        relinked["correspondence"]["reconstruction_inputs"]["sources"],
        reordered["reconstruction_inputs"]["sources"]
    );
    assert_eq!(
        relinked["correspondence"]["node_correspondences"],
        nominal_request["node_correspondences"]
    );

    // Unknown request members are refused at the wire, not ignored.
    let mut unknown = all_request["reconstruction_inputs"].clone();
    unknown["future"] = json!(true);
    assert!(
        serde_json::from_value::<CheckedPackageMigrationRequest>(json!({
            "target_package_id": all_request["target_package_id"],
            "reconstruction_inputs": unknown,
        }))
        .is_err()
    );
}
