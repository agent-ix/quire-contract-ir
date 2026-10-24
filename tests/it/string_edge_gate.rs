// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-056 consumer controls for the authoritative string-edge scanner.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use ix_trace_rs::trace;

static NEXT_SCRATCH: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str, source: &str) -> Self {
        let serial = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "quire-contract-ir-fr056-{}-{serial}-{label}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("unique scratch directory");
        fs::create_dir_all(root.join("src")).expect("first source root");
        fs::create_dir_all(root.join("crates/quire-contract-model/src"))
            .expect("second source root");
        fs::write(root.join("src/lib.rs"), source).expect("first source");
        fs::write(
            root.join("crates/quire-contract-model/src/lib.rs"),
            "pub enum Kind { Closed }\n",
        )
        .expect("second source");
        fs::write(root.join("string-edge-rules.json"), "{\"allow_list\":[]}")
            .expect("strict empty rules");
        Self(root)
    }

    fn run(&self, scanner: &Path) -> Output {
        Command::new("make")
            .arg("--no-print-directory")
            .arg("-C")
            .arg(&self.0)
            .arg("-f")
            .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Makefile"))
            .arg("string-edge")
            .arg(format!("STRING_EDGE={}", scanner.display()))
            .output()
            .expect("make invokes the scanner target")
    }

    fn write(&self, path: &str, contents: &str) {
        fs::write(self.0.join(path), contents).expect("write controlled consumer file");
    }

    fn lint_plan(&self) -> Output {
        Command::new("make")
            .arg("-n")
            .arg("--no-print-directory")
            .arg("-C")
            .arg(&self.0)
            .arg("-f")
            .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Makefile"))
            .arg("lint")
            .output()
            .expect("make lint dry run")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove own scratch directory");
    }
}

fn scanner() -> PathBuf {
    std::env::var_os("QSL_STRING_EDGE_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("qsl-string-edge"))
}

fn diagnostic(output: &Output) -> String {
    format!(
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_scan_status(output: &Output, expected: i32, diagnostic_fragment: &str) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        diagnostic(output)
    );
    let report = diagnostic(output);
    assert!(
        report.contains(diagnostic_fragment),
        "expected {diagnostic_fragment:?}: {report}"
    );
    if expected == 2 {
        assert!(!report.contains("string-edge: clean"), "{report}");
    }
}

#[test]
#[trace("FR-056-AC-1", "FR-056-AC-2", "FR-056-AC-5")]
fn normal_gate_uses_both_roots_and_refuses_unmarked_dispatch() {
    let clean = Scratch::new(
        "clean",
        "pub enum Kind { Accepted, Other }\npub fn accepted(kind: Kind) -> bool { matches!(kind, Kind::Accepted) }\n",
    );
    let plan = clean.lint_plan();
    assert!(plan.status.success(), "{}", diagnostic(&plan));
    let plan_text = String::from_utf8_lossy(&plan.stdout);
    assert!(plan_text.contains("qsl-string-edge --base-dir ."));
    assert!(plan_text.contains("cargo clippy"));

    let output = clean.run(&scanner());
    assert!(output.status.success(), "{}", diagnostic(&output));
    let invocation = String::from_utf8_lossy(&output.stdout);
    for argument in [
        "--base-dir .",
        "--crate-root src",
        "--crate-root crates/quire-contract-model/src",
        "--rules string-edge-rules.json",
    ] {
        assert!(invocation.contains(argument), "{argument}: {invocation}");
    }

    let unmarked = Scratch::new(
        "unmarked",
        "pub fn accepted(kind: &str) -> bool { match kind { \"accepted\" => true, _ => false } }\n",
    );
    let output = unmarked.run(&scanner());
    assert_scan_status(&output, 2, "Error 1");
    assert!(diagnostic(&output).contains("src/lib.rs:1 in accepted"));

    clean.write(
        "src/lib.rs",
        "pub fn accepted(kind: &str) -> bool { match kind { \"node\" => true, _ => false } }\n",
    );
    assert_scan_status(&clean.run(&scanner()), 2, "src/lib.rs:1 in accepted");
    clean.write("src/lib.rs", "pub enum Kind { Node, Other }\npub fn accepted(kind: Kind) -> bool { matches!(kind, Kind::Node) }\n");
    assert!(clean.run(&scanner()).status.success());

    let output = clean.run(&clean.0.join("missing-scanner"));
    assert!(!output.status.success(), "{}", diagnostic(&output));
}

#[test]
#[trace("FR-056-AC-3")]
fn genuine_intake_marker_compiles_and_only_exempts_its_function() {
    let source = "#[ix_trace_rs::string_edge]\n\
        pub fn decode(s: &str) -> Option<Kind> { match s { \"node\" => Some(Kind::Node), _ => None } }\n\
        pub enum Kind { Node }\n\
        pub fn adverse(s: &str) -> bool { s == \"node\" }\n";
    let fixture = Scratch::new("marked", source);
    assert_scan_status(&fixture.run(&scanner()), 2, "src/lib.rs:4 in adverse");
    fixture.write(
        "src/lib.rs",
        &source.replace("pub fn adverse(s: &str) -> bool { s == \"node\" }\n", ""),
    );
    assert!(fixture.run(&scanner()).status.success());

    let metadata = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("resolve neutral marker");
    assert!(metadata.status.success(), "{}", diagnostic(&metadata));
    let json: serde_json::Value = serde_json::from_slice(&metadata.stdout).expect("metadata JSON");
    let marker_manifest = json["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .find(|package| package["name"] == "ix-trace-rs")
        .and_then(|package| package["manifest_path"].as_str())
        .expect("neutral marker manifest");
    let marker_dir = Path::new(marker_manifest)
        .parent()
        .expect("marker directory");
    fixture.write(
        "Cargo.toml",
        &format!(
            "[package]\nname = \"fr056-marker-consumer\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nix-trace-rs = {{ path = {:?} }}\n[workspace]\n",
            marker_dir
        ),
    );
    fixture.write(
        "src/main.rs",
        "fn main() { assert!(fr056_marker_consumer::decode(\"node\").is_some()); assert!(fr056_marker_consumer::decode(\"other\").is_none()); }\n",
    );
    let compiled = Command::new("cargo")
        .args(["run", "--offline", "--quiet"])
        .current_dir(&fixture.0)
        .env("CARGO_TARGET_DIR", fixture.0.join("target"))
        .output()
        .expect("compile genuine marked intake");
    assert!(compiled.status.success(), "{}", diagnostic(&compiled));

    for marker in ["#[other::string_edge]", "// #[ix_trace_rs::string_edge]"] {
        fixture.write(
            "src/lib.rs",
            &format!("{marker}\npub fn adverse(s: &str) -> bool {{ s == \"node\" }}\n"),
        );
        assert_scan_status(&fixture.run(&scanner()), 2, "src/lib.rs:2 in adverse");
    }
}

#[test]
#[trace("FR-056-AC-4")]
fn open_keys_and_strict_nonbranching_rules() {
    let fixture = Scratch::new(
        "open-keys",
        "pub fn same_key(left: &str, right: &str) -> bool { left == right }\n",
    );
    assert!(fixture.run(&scanner()).status.success());
    fixture.write(
        "src/lib.rs",
        "pub fn same_key(left: &str, right: &str) -> bool { left == right }\n\
         pub fn display(value: &str) -> bool { value == \"preview\" }\n",
    );
    assert_scan_status(&fixture.run(&scanner()), 2, "src/lib.rs:2 in display");
    fixture.write(
        "string-edge-rules.json",
        r#"{"allow_list":[{"file":"src/lib.rs","item":"display","reason":"Result is returned for display only"}]}"#,
    );
    assert!(fixture.run(&scanner()).status.success());
    fixture.write(
        "src/lib.rs",
        "pub fn same_key(left: &str, right: &str) -> bool { left == right }\n\
         pub fn display(value: &str) -> bool { if value == \"node\" { true } else { false } }\n",
    );
    assert_scan_status(&fixture.run(&scanner()), 2, "gates a branch");
}

#[test]
#[trace("FR-056-AC-5")]
fn normal_prerequisite_propagates_command_and_input_failures() {
    let fixture = Scratch::new("errors", "pub enum Kind { Node }\n");
    let binary = scanner();
    let missing = fixture.run(&fixture.0.join("missing-scanner"));
    assert!(!missing.status.success(), "{}", diagnostic(&missing));
    fixture.write("string-edge-rules.json", "{");
    assert_scan_status(&fixture.run(&binary), 2, "string-edge:");
    fixture.write("string-edge-rules.json", "{\"allow_list\":[]}");
    fs::remove_dir_all(fixture.0.join("crates/quire-contract-model/src"))
        .expect("remove controlled root");
    assert_scan_status(&fixture.run(&binary), 2, "string-edge:");
    fs::create_dir_all(fixture.0.join("crates/quire-contract-model/src"))
        .expect("restore controlled root");
    fixture.write("src/lib.rs", "pub fn invalid( {\n");
    assert_scan_status(&fixture.run(&binary), 2, "string-edge:");
}

#[test]
#[trace("FR-056-AC-6")]
fn marked_intake_depends_on_neutral_trace_without_scanner_or_qsl_dependency() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
            "--offline",
        ])
        .current_dir(&root)
        .output()
        .expect("cargo metadata runs");
    assert!(output.status.success(), "{}", diagnostic(&output));
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata JSON");
    let packages = metadata["packages"].as_array().expect("packages");
    for package_name in ["quire-contract-ir", "quire-contract-model"] {
        let package = packages
            .iter()
            .find(|package| package["name"] == package_name)
            .expect("production package");
        let deps = package["dependencies"].as_array().expect("dependencies");
        assert!(
            deps.iter().any(|dependency| {
                dependency["name"] == "ix-trace-rs" && dependency["kind"].is_null()
            }),
            "each marked production crate must resolve the neutral marker as a normal dependency"
        );
        assert!(
            deps.iter().all(|dependency| {
                !matches!(
                    dependency["name"].as_str(),
                    Some("quire-spec-language" | "qsl-string-edge")
                )
            }),
            "{package_name} cannot acquire QSL or scanner as a Cargo dependency"
        );
    }
}
