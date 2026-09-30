use std::{path::PathBuf, process::Command};

use ix_trace_rs::trace;
use serde_json::Value;

fn metadata() -> Value {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .expect("cargo metadata must start");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata must emit JSON")
}

fn package<'a>(metadata: &'a Value, name: &str) -> &'a Value {
    metadata["packages"]
        .as_array()
        .expect("metadata packages must be an array")
        .iter()
        .find(|package| package["name"] == name)
        .unwrap_or_else(|| panic!("metadata must contain {name}"))
}

#[trace("TC-041", "FR-028-AC-1", "FR-028-AC-3", "FR-028-AC-4")]
#[test]
fn tc_041_model_dependency_graph_is_cycle_free_and_owner_free() {
    let workspace = metadata();
    let model = package(&workspace, "quire-contract-model");

    let model_dependencies = model["dependencies"]
        .as_array()
        .expect("model dependencies must be an array");
    let forbidden = [
        "quire-contract-ir",
        "quire-spec-language",
        "quire-observation",
        "quire-protocol",
        "tl-syntax",
        "tl-parse",
        "tl-mltl",
        "tl-rewrite",
        "quire-mltl",
    ];
    for dependency in model_dependencies {
        let name = dependency["name"]
            .as_str()
            .expect("dependency names must be strings");
        assert!(
            !forbidden.contains(&name),
            "cycle-free model contains forbidden production dependency {name}"
        );
        assert_eq!(dependency["optional"], false);
    }

    assert_spec_language_production_graph_excludes(&workspace, "quire-contract-ir");
}

/// Walk the resolved graph from every `quire-spec-language` node over normal
/// (non-dev, non-build) dependency edges and assert `target` is unreachable.
fn assert_spec_language_production_graph_excludes(metadata: &Value, target: &str) {
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .expect("resolve nodes must be an array");
    let name_of = |id: &str| -> String {
        let package = metadata["packages"]
            .as_array()
            .expect("metadata packages must be an array")
            .iter()
            .find(|package| package["id"] == id)
            .unwrap_or_else(|| panic!("metadata must contain package {id}"));
        package["name"]
            .as_str()
            .expect("package names must be strings")
            .to_owned()
    };
    let mut pending: Vec<String> = nodes
        .iter()
        .filter_map(|node| node["id"].as_str())
        .filter(|id| name_of(id) == "quire-spec-language")
        .map(str::to_owned)
        .collect();
    assert!(
        !pending.is_empty(),
        "graph must contain quire-spec-language"
    );
    let mut seen = std::collections::BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        assert_ne!(
            name_of(&id),
            target,
            "{target} is reachable from quire-spec-language over production edges"
        );
        let node = nodes
            .iter()
            .find(|node| node["id"] == id.as_str())
            .unwrap_or_else(|| panic!("resolve must contain node {id}"));
        for dependency in node["deps"].as_array().expect("node deps must be an array") {
            let normal = dependency["dep_kinds"]
                .as_array()
                .expect("dep_kinds must be an array")
                .iter()
                .any(|kind| kind["kind"].is_null());
            if normal {
                pending.push(
                    dependency["pkg"]
                        .as_str()
                        .expect("dependency pkg must be a string")
                        .to_owned(),
                );
            }
        }
    }
}

fn accepts_model_version(_: quire_contract_model::SchemaVersion) {}

fn accepts_bridge_version(_: quire_contract_ir::SchemaVersion) {}

#[trace("TC-041", "FR-028-AC-2", "FR-028-AC-5")]
#[test]
fn tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single() {
    accepts_model_version(quire_contract_ir::SchemaVersion::V1_1);
    accepts_bridge_version(quire_contract_model::SchemaVersion::V1_1);
    assert_eq!(
        quire_contract_ir::CANONICAL_PROFILE,
        quire_contract_model::CANONICAL_PROFILE
    );
}
