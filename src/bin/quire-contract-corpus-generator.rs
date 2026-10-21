// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Regenerate the checked-in conformance corpus from declared Rust fixture builders.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

#[path = "quire-contract-corpus-generator/ordered_json.rs"]
mod ordered_json;
use ordered_json::J;

macro_rules! obj {
    ($($key:expr => $value:expr),* $(,)?) => {
        J::Object(vec![$(($key.to_owned(), J::from($value))),*])
    };
}

#[path = "quire-contract-corpus-generator/cases.rs"]
mod cases;

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("corpus generation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute() -> io::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut corpus = root.join("corpus/contract-v0.1");
    let mut runner = default_runner(&root);
    let mut arguments = env::args_os().skip(1);
    while let Some(flag) = arguments.next() {
        let value = arguments.next().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,
                "expected --output DIR or --runner FILE",
            )
        })?;
        match flag.to_str() {
            Some("--output") => corpus = value.into(),
            Some("--runner") => runner = value.into(),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "unknown generator option",
                ))
            }
        }
    }
    regenerate(&root, &corpus, &runner)
}

struct Stage(PathBuf);

impl Drop for Stage {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn unused_sibling(destination: &Path, kind: &str) -> io::Result<PathBuf> {
    let parent = destination
        .parent()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "output has no parent"))?;
    let name = destination
        .file_name()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "output has no name"))?
        .to_string_lossy();
    for attempt in 0..128 {
        let candidate = parent.join(format!(".{name}.{kind}-{}-{attempt}", std::process::id()));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(Error::new(
        ErrorKind::AlreadyExists,
        "no free corpus staging path",
    ))
}

fn regenerate(root: &Path, corpus: &Path, runner: &Path) -> io::Result<()> {
    let parent = corpus
        .parent()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "output has no parent"))?;
    fs::create_dir_all(parent)?;
    let stage = Stage(unused_sibling(corpus, "stage")?);
    fs::create_dir(&stage.0)?;
    let staged_corpus = stage.0.join(
        corpus
            .file_name()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "output has no name"))?,
    );
    fs::create_dir(&staged_corpus)?;
    fs::copy(
        root.join("corpus/contract-v0.1/README.md"),
        staged_corpus.join("README.md"),
    )?;
    generate(root, &staged_corpus, runner)?;

    if corpus.exists() {
        let backup = unused_sibling(corpus, "backup")?;
        fs::rename(corpus, &backup)?;
        if let Err(error) = fs::rename(&staged_corpus, corpus) {
            fs::rename(&backup, corpus)?;
            return Err(error);
        }
        fs::remove_dir_all(backup)?;
    } else {
        fs::rename(&staged_corpus, corpus)?;
    }
    Ok(())
}

fn default_runner(root: &Path) -> PathBuf {
    let target = env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    let target = if target.is_absolute() {
        target
    } else {
        root.join(target)
    };
    target.join("debug/quire-contract-conformance")
}

fn write_json(path: &Path, value: &J) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value).map_err(Error::other)?;
    bytes.push(b'\n');
    fs::write(path, bytes)
}

fn run(runner: &Path, corpus: &Path, schemas: &Path) -> io::Result<Vec<J>> {
    let output = Command::new(runner)
        .args(["run", "--corpus"])
        .arg(corpus)
        .arg("--schemas")
        .arg(schemas)
        .output()?;
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(Error::other(format!(
            "runner failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice::<J>(line).map_err(Error::other))
        .collect()
}

fn placeholder(operation: &str) -> J {
    match operation {
        "package" | "expression" => obj!(
            "valid" => false,
            "diagnostics" => Vec::<J>::new(),
            "canonical" => Vec::<J>::new(),
            "dependencies" => Vec::<J>::new()
        ),
        "coverage" => obj!("diagnostics" => Vec::<J>::new(), "coverage" => J::Null),
        _ => unreachable!("case builder emits only three operations"),
    }
}

fn generate(root: &Path, corpus: &Path, runner: &Path) -> io::Result<()> {
    let cases = cases::build_cases();
    for directory in ["inputs", "expectations", "canonical"] {
        fs::create_dir_all(corpus.join(directory))?;
    }
    let mut declared = BTreeMap::<String, BTreeSet<String>>::new();
    for case in &cases {
        write_json(
            &corpus.join(format!("inputs/{}.json", case.id)),
            &case.value,
        )?;
        write_json(
            &corpus.join(format!("expectations/{}.json", case.id)),
            &placeholder(case.operation),
        )?;
        declared.insert(case.id.clone(), case.covers.clone());
    }
    let rows = run(runner, corpus, &root.join("schemas"))?;
    if rows.len() != cases.len() {
        return Err(Error::other(format!(
            "runner returned {} rows for {} fixtures",
            rows.len(),
            cases.len()
        )));
    }
    for mut row in rows {
        let fixture_id = row
            .get("fixture_id")
            .and_then(J::as_str)
            .ok_or_else(|| Error::other("runner row omitted fixture_id"))?
            .to_owned();
        let observed = row
            .get("covers")
            .and_then(J::as_array)
            .ok_or_else(|| Error::other("runner row omitted covers"))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| Error::other("invalid cover token"))
            })
            .collect::<io::Result<BTreeSet<_>>>()?;
        let missing = declared
            .get(&fixture_id)
            .ok_or_else(|| Error::other("runner returned an undeclared fixture"))?
            .difference(&observed)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(Error::other(format!(
                "{fixture_id} does not exercise {missing:?}"
            )));
        }
        let mut actual = row
            .remove("actual")
            .ok_or_else(|| Error::other("runner row omitted actual"))?;
        if let Some(canonical) = actual.get_mut("canonical") {
            let items = canonical
                .as_array_mut()
                .ok_or_else(|| Error::other("invalid canonical array"))?;
            for (index, canonical) in items.iter_mut().enumerate() {
                let bytes = canonical
                    .remove("bytes")
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .ok_or_else(|| Error::other("canonical result omitted bytes"))?;
                let relative = format!("canonical/{fixture_id}-{index}.json");
                fs::write(corpus.join(&relative), bytes.as_bytes())?;
                canonical.set("bytes_path", relative);
            }
        }
        write_json(
            &corpus.join(format!("expectations/{fixture_id}.json")),
            &actual,
        )?;
    }
    if run(runner, corpus, &root.join("schemas"))?
        .iter()
        .any(|row| row.get("status").and_then(J::as_str) != Some("match"))
    {
        return Err(Error::other(
            "regenerated corpus does not match its own expectations",
        ));
    }
    Ok(())
}
