use std::{path::PathBuf, process::Command};

use ix_trace_rs::trace;
use serde_json::Value;

fn metadata() -> Value {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--locked",
            "--offline",
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

    // The root package depends on the model and on no crate from the
    // quire-spec-language repository, in any dependency kind, by name and by source.
    let root = package(&workspace, "quire-contract-ir");
    for dependency in root["dependencies"]
        .as_array()
        .expect("root dependencies must be an array")
    {
        let name = dependency["name"]
            .as_str()
            .expect("dependency names must be strings");
        assert!(
            name != "quire-spec-language" && !name.starts_with("qsl-"),
            "root package depends on QSL crate {name}"
        );
        let source = dependency["source"].as_str().unwrap_or("");
        assert!(
            !source.starts_with("git+https://github.com/agent-ix/quire-spec-language"),
            "root package depends on {name} from the quire-spec-language repository: {source}"
        );
    }
    assert!(root["dependencies"]
        .as_array()
        .expect("root dependencies must be an array")
        .iter()
        .any(|dependency| dependency["name"] == "quire-contract-model"));
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
