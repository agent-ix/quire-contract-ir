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
    assert!(!output.status.success(), "{}", diagnostic(&output));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("src/lib.rs:1 in accepted"),
        "the real scanner must locate the unmarked item: {}",
        diagnostic(&output)
    );

    let output = clean.run(&clean.0.join("missing-scanner"));
    assert!(!output.status.success(), "{}", diagnostic(&output));
}
