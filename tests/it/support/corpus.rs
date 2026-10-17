// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Conformance-corpus scratch helpers shared by the TC-018 test modules.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::Value;

pub fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn corpus() -> PathBuf {
    repository().join("corpus/contract-v0.1")
}

pub fn runner(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
        .args(arguments)
        .output()
        .unwrap()
}

pub fn schemas() -> PathBuf {
    repository().join("schemas")
}

pub fn run_with(corpus: &Path, schemas: &Path) -> Output {
    runner(&[
        "run",
        "--corpus",
        corpus.to_str().unwrap(),
        "--schemas",
        schemas.to_str().unwrap(),
    ])
}

pub fn run_corpus(path: &Path) -> Output {
    run_with(path, &schemas())
}

/// A scratch copy of the corpus at `<root>/contract-v0.1` with the schemas it
/// runs against at `<root>/schemas`, so a test can change either.
pub struct Scratch(pub PathBuf);

impl Scratch {
    pub fn corpus(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "quire-contract-ir-tc018-{}-{label}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        let path = root.join("contract-v0.1");
        copy_tree(&corpus(), &path);
        copy_tree(&schemas(), &root.join("schemas"));
        Self(path)
    }

    pub fn root(&self) -> &Path {
        self.0.parent().unwrap()
    }

    pub fn schemas(&self) -> PathBuf {
        self.root().join("schemas")
    }

    pub fn run(&self) -> Output {
        run_with(&self.0, &self.schemas())
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}

pub fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

pub fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

pub fn rows(output: &Output) -> Vec<Value> {
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| serde_json::from_slice::<Value>(row).unwrap())
        .collect()
}
