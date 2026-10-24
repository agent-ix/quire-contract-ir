// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-055: the ordinary Make recipe consumes the released owner checker.

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
    fn new(label: &str, lock: &str) -> Self {
        let serial = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "quire-contract-ir-fr055-{}-{serial}-{label}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("unique scratch directory");
        fs::write(path.join("Cargo.lock"), lock).expect("scratch lock");
        Self(path)
    }

    fn run_deny(&self, checker: &Path) -> Output {
        run_deny_at(&self.0, checker)
    }
}

fn run_deny_at(directory: &Path, checker: &Path) -> Output {
    Command::new("make")
        .arg("--no-print-directory")
        .arg("-C")
        .arg(directory)
        .arg("-f")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Makefile"))
        .arg("deny")
        // Isolate cargo-deny's unrelated policy stage. The checker is the
        // real Quire executable and is never replaced or reimplemented.
        .arg("CARGO=true")
        .arg(format!("QUIRE={}", checker.display()))
        .output()
        .expect("make deny runs")
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove own scratch directory");
    }
}

fn checker() -> PathBuf {
    std::env::var_os("QUIRE_LOCK_CHECKER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("quire"))
}

fn source(revision: u8) -> String {
    // Synthetic full fragments make CLI's source parser reach its coherence
    // rule; these are adverse inputs, not tracked dependency revisions.
    format!("git+https://github.com/agent-ix/quire-rs?branch=main#{revision:040x}")
}

fn lock(packages: &[(&str, u8)]) -> String {
    let mut text = String::from("version = 3\n");
    for (name, revision) in packages {
        text.push_str(&format!(
            "\n[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\nsource = \"{}\"\n",
            source(*revision)
        ));
    }
    text
}

fn diagnostic(output: &Output) -> String {
    format!(
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
#[trace("FR-055-AC-1", "FR-055-AC-2", "FR-055-AC-3")]
fn normal_deny_recipe_propagates_the_owned_lock_checker() {
    let checker = checker();
    let aligned = Scratch::new("aligned", &lock(&[("alpha", 1), ("beta", 1)]));
    let output = aligned.run_deny(&checker);
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("check-lock Cargo.lock"),
        "the normal recipe must invoke the real checker: {}",
        diagnostic(&output)
    );

    for (label, packages) in [
        ("duplicate", vec![("alpha", 1), ("alpha", 2)]),
        ("split", vec![("alpha", 1), ("beta", 2)]),
    ] {
        let scratch = Scratch::new(label, &lock(&packages));
        let output = scratch.run_deny(&checker);
        assert!(!output.status.success(), "{label}: {}", diagnostic(&output));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("coherence refused"),
            "{label} must reach the owner's coherence rule: {}",
            diagnostic(&output)
        );
    }

    let malformed = Scratch::new("malformed", "version = [\n");
    let output = malformed.run_deny(&checker);
    assert!(!output.status.success(), "{}", diagnostic(&output));
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("coherence refused"),
        "malformed input is distinct from a coherence refusal: {}",
        diagnostic(&output)
    );

    let missing = Scratch::new("missing", &lock(&[("alpha", 1)]));
    let output = missing.run_deny(&missing.0.join("missing-checker"));
    assert!(!output.status.success(), "{}", diagnostic(&output));
}

#[test]
#[trace("FR-055-AC-1", "FR-055-AC-5")]
fn committed_lock_is_checked_without_rewriting_the_dependency_graph() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let paths = [
        root.join("Cargo.toml"),
        root.join("crates/quire-contract-model/Cargo.toml"),
        root.join("Cargo.lock"),
    ];
    let before = paths
        .iter()
        .map(|path| fs::read(path).expect("committed dependency file"))
        .collect::<Vec<_>>();
    let output = run_deny_at(&root, &checker());
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("check-lock Cargo.lock"),
        "committed lock must reach the owner command: {}",
        diagnostic(&output)
    );
    for (path, original) in paths.iter().zip(before) {
        assert_eq!(
            fs::read(path).expect("dependency file after check"),
            original
        );
    }
}
