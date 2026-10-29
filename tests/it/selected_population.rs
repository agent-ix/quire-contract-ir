// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use crate::support::checked_package::{mint_ungrouped_structural_keys, refresh_identity};
use ix_trace_rs::trace;
use quire_contract_model::{
    selected_population::{
        select_population, CheckedSelectedPopulationOutcome as Outcome,
        CheckedSelectedPopulationSelector as Selector, CheckedSelectedPopulationSource as Source,
    },
    CheckedOccurrenceRole, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn qsl_fixture() -> Scratch {
    let qsl = std::env::var_os("QUIRE_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("selected-population test requires QUIRE_SPEC_LANGUAGE_DIR");
    let scratch_path =
        std::env::temp_dir().join(format!("ir760-duplicate-max-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch_path);
    fs::create_dir(&scratch_path).expect("create QSL producer scratch");
    let output = scratch_path.join("emitted");
    let target = std::env::var_os("QUIRE_SPEC_LANGUAGE_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/qsl-producer"));
    let result = Command::new("cargo")
        .arg("run")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(qsl.join("Cargo.toml"))
        .arg("--example")
        .arg("population_v2_fixtures")
        .arg("--features")
        .arg("population-fixtures")
        .arg("--")
        .arg(&output)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("run authoritative QSL producer");
    assert!(
        result.status.success(),
        "QSL producer failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Scratch(scratch_path)
}

fn admitted(package: &Value, domain: &Value) -> CheckedPackageV2 {
    let domain_bytes = quire_canonical::to_vec(domain, quire_canonical::Limits::new(1 << 30))
        .expect("QSL domain package is canonicalizable");
    let mut evidence = CheckedPackageEvidence::new();
    for root in package["semantic_graph"]["nodes"]
        .as_array()
        .expect("QSL graph nodes")
        .iter()
        .filter(|node| node["semantic_form"] == "model_root")
    {
        let identity = root["owner"]["identity"].as_str().expect("model identity");
        evidence
            .insert_domain_package_document(identity, domain_bytes.clone())
            .expect("selected domain package evidence");
    }
    evidence.support_feature("quire.value.complete/v1");
    let bytes = quire_canonical::to_vec(package, quire_canonical::Limits::new(1 << 30))
        .expect("QSL package is canonicalizable");
    match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("rekeyed duplicate-max package must admit: {other:?}"),
    }
}

#[trace("FR-053-AC-3", "TC-453")]
#[test]
fn duplicate_max_on_reader_admitted_population_is_not_population() {
    let scratch = qsl_fixture();
    let domain: Value = serde_json::from_slice(
        &fs::read(scratch.0.join("emitted/spine-model.semantic-ir.json"))
            .expect("QSL domain package output"),
    )
    .expect("QSL domain package JSON");
    let mut package: Value = serde_json::from_slice(
        &fs::read(scratch.0.join("emitted/population-2-5.json"))
            .expect("QSL emitted population package"),
    )
    .expect("QSL package JSON");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("semantic graph nodes");
    let population = nodes
        .iter_mut()
        .find(|node| {
            node["node_tag"] == "bounded_domain" && node["semantic_form"] == "model_population"
        })
        .expect("QSL emitted population node");
    let maximum = population["body"]["members"]
        .as_array()
        .expect("population members")
        .iter()
        .find(|member| member["name"] == "max")
        .expect("original max binding")
        .clone();
    population["body"]["members"]
        .as_array_mut()
        .expect("population members")
        .push(maximum);

    mint_ungrouped_structural_keys(&mut package);
    refresh_identity(&mut package);
    let package = admitted(&package, &domain);
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "bounded_domain"
                && node.semantic_form.as_ref() == "model_population"
        })
        .expect("rekeyed package retained its population");
    let occurrence = node
        .occurrences
        .iter()
        .find(|occurrence| occurrence.role == CheckedOccurrenceRole::Type)
        .expect("population Type occurrence")
        .clone();
    let result = select_population(Source::Admitted {
        package: &package,
        selector: Selector {
            package_id: package.package_id().clone(),
            binding: node.node_id.clone(),
            occurrence,
        },
    });
    assert!(
        matches!(result, Outcome::NotPopulation(_)),
        "duplicate maximum must refuse projection without an extent: {result:?}"
    );
}
