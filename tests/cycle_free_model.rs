use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use ix_trace_rs::trace;
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn metadata(manifest: &Path) -> Value {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--locked",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
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

#[trace("TC-041", "FR-028-AC-1", "FR-028-AC-4")]
#[test]
fn tc_041_model_dependency_graph_is_cycle_free_and_owner_free() {
    let repository = root();
    let workspace = metadata(&repository.join("Cargo.toml"));
    let model = package(&workspace, "quire-contract-model");
    let bridge = package(&workspace, "quire-contract-ir");

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
    ];
    for dependency in model_dependencies {
        let name = dependency["name"]
            .as_str()
            .expect("dependency names must be strings");
        assert!(
            !forbidden.contains(&name),
            "cycle-free model contains forbidden production dependency {name}"
        );
        assert_eq!(dependency["kind"], Value::Null);
        assert_eq!(dependency["optional"], false);
    }
    assert_eq!(
        model["features"].as_object().map(|features| features.len()),
        Some(0)
    );

    let bridge_dependencies = bridge["dependencies"]
        .as_array()
        .expect("bridge dependencies must be an array");
    assert!(bridge_dependencies.iter().any(|dependency| {
        dependency["name"] == "quire-contract-model" && dependency["kind"] == Value::Null
    }));
    assert_eq!(
        bridge["features"]
            .as_object()
            .map(|features| features.len()),
        Some(0)
    );

    let workspace_members = workspace["workspace_members"]
        .as_array()
        .expect("workspace members must be an array");
    assert_eq!(workspace_members.len(), 2);

    let makefile = fs::read_to_string(repository.join("Makefile"))
        .expect("workspace release gates must be readable");
    for gate in [
        "clippy --locked --workspace --all-targets -- -D warnings",
        "test --locked --workspace --all-targets",
        "check --locked --workspace --all-targets",
        "build --locked --workspace --release",
    ] {
        assert!(
            makefile.contains(gate),
            "workspace release gate lacks {gate}"
        );
    }
}

fn accepts_model_version(_: quire_contract_model::SchemaVersion) {}

fn accepts_bridge_version(_: quire_contract_ir::SchemaVersion) {}

#[trace("TC-041", "FR-028-AC-2", "FR-028-AC-5")]
#[test]
fn tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single() {
    accepts_model_version(quire_contract_ir::SchemaVersion::V1_0);
    accepts_bridge_version(quire_contract_model::SchemaVersion::V1_0);
    assert_eq!(
        quire_contract_ir::CANONICAL_PROFILE,
        quire_contract_model::CANONICAL_PROFILE
    );
    assert_eq!(
        quire_contract_ir::hex_digest(b"cycle-free-model"),
        quire_contract_model::hex_digest(b"cycle-free-model")
    );

    let repository = root();
    for module in [
        "binding.rs",
        "canonical.rs",
        "conformance.rs",
        "coverage.rs",
        "expression.rs",
        "identity.rs",
        "limits.rs",
        "wire.rs",
    ] {
        assert!(repository
            .join("crates/quire-contract-model/src")
            .join(module)
            .is_file());
        assert!(
            !repository.join("src").join(module).exists(),
            "semantic module {module} must have one owning source"
        );
    }

    let bridge = fs::read_to_string(repository.join("src/lib.rs"))
        .expect("compatibility bridge source must be readable");
    assert!(bridge.contains("pub use quire_contract_model::*;"));
}

#[trace("TC-041", "FR-028-AC-3")]
#[test]
fn tc_041_existing_contract_ir_imports_build_through_the_model_package_alias() {
    let repository = root();
    let manifest = repository.join("tests/fixtures/model-alias-consumer/Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "check",
            "--locked",
            "--offline",
            "--all-targets",
            "--manifest-path",
        ])
        .arg(&manifest)
        .arg("--target-dir")
        .arg(repository.join("target/tc-041-model-alias"))
        .output()
        .expect("aliased model consumer check must start");
    assert!(
        output.status.success(),
        "aliased model consumer failed to build: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
