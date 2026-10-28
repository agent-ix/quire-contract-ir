// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Cross-platform corpus capture and direct comparison for the named TC-019 CI lane.

use std::{collections::BTreeMap, env, fs, path::Path, process::ExitCode};

use quire_contract_model::{run_corpus, FixtureStatus, CANONICAL_PROFILE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SYSTEMS: [&str; 3] = ["linux", "macos", "windows"];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct CanonicalEntry {
    identity: String,
    kind: String,
    bytes: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticEntry {
    code: String,
    path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureEntry {
    id: String,
    valid: Option<bool>,
    canonical: Vec<CanonicalEntry>,
    diagnostics: Vec<DiagnosticEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    run_id: u64,
    attempt: u32,
    os: String,
    profile: String,
    fixtures: Vec<FixtureEntry>,
}

fn require_string<'a>(object: &'a Value, field: &str) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing or invalid {field}"))
}

fn project_row(row: &Value) -> Result<FixtureEntry, String> {
    let id = require_string(row, "fixture_id")?.to_owned();
    if row.get("status").and_then(Value::as_str) != Some("match") {
        return Err(format!(
            "fixture {id} does not match its checked-in expectation"
        ));
    }
    let actual = row
        .get("actual")
        .ok_or_else(|| format!("fixture {id} has no actual result"))?;
    let valid = match actual.get("valid") {
        Some(value) => Some(
            value
                .as_bool()
                .ok_or_else(|| format!("fixture {id} has invalid valid field"))?,
        ),
        None => None,
    };
    let canonical = actual
        .get("canonical")
        .map(|value| {
            value
                .as_array()
                .ok_or_else(|| format!("fixture {id} has invalid canonical array"))?
                .iter()
                .map(|item| {
                    Ok(CanonicalEntry {
                        identity: require_string(item, "identity")?.to_owned(),
                        kind: require_string(item, "kind")?.to_owned(),
                        bytes: require_string(item, "bytes")?.to_owned(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?
        .unwrap_or_default();
    let diagnostics = actual
        .get("diagnostics")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("fixture {id} has no diagnostic array"))?
        .iter()
        .map(|item| {
            Ok(DiagnosticEntry {
                code: require_string(item, "code")?.to_owned(),
                path: require_string(item, "path")?.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if valid == Some(false) && diagnostics.is_empty() {
        return Err(format!("rejected fixture {id} has no diagnostics"));
    }
    if valid == Some(false) && !canonical.is_empty() {
        return Err(format!("rejected fixture {id} has canonical output"));
    }
    Ok(FixtureEntry {
        id,
        valid,
        canonical,
        diagnostics,
    })
}

fn capture(
    corpus: &Path,
    schemas: &Path,
    run_id: u64,
    attempt: u32,
    os: &str,
) -> Result<Capture, String> {
    if !SYSTEMS.contains(&os) || run_id == 0 || attempt == 0 {
        return Err("invalid OS, run ID, or attempt".to_owned());
    }
    if os != env::consts::OS {
        return Err(format!(
            "capture OS {os} differs from executing host {}",
            env::consts::OS
        ));
    }
    let first = run_corpus(corpus, schemas).map_err(|error| format!("corpus run: {error:?}"))?;
    let second = run_corpus(corpus, schemas).map_err(|error| format!("repeat run: {error:?}"))?;
    if first.len() != second.len() {
        return Err("repeat run changed fixture count".to_owned());
    }
    let mut fixtures = Vec::with_capacity(first.len());
    for (first, second) in first.iter().zip(&second) {
        if first.status() != FixtureStatus::Match || second.status() != FixtureStatus::Match {
            return Err("corpus result differs from a checked-in expectation".to_owned());
        }
        let first = serde_json::to_value(first).map_err(|error| error.to_string())?;
        let second = serde_json::to_value(second).map_err(|error| error.to_string())?;
        let projected = project_row(&first)?;
        if projected != project_row(&second)? {
            return Err(format!("local repeat changed fixture {}", projected.id));
        }
        fixtures.push(projected);
    }
    let result = Capture {
        run_id,
        attempt,
        os: os.to_owned(),
        profile: CANONICAL_PROFILE.to_owned(),
        fixtures,
    };
    validate_capture(&result, corpus)?;
    Ok(result)
}

fn fixture_universe(corpus: &Path) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();
    for entry in fs::read_dir(corpus.join("inputs")).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .and_then(|name| name.strip_suffix(".json"))
            .ok_or_else(|| "non-JSON corpus input".to_owned())?;
        ids.push(name.to_owned());
    }
    ids.sort();
    if ids.is_empty() || ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("empty or duplicate corpus input inventory".to_owned());
    }
    Ok(ids)
}

fn validate_capture(capture: &Capture, corpus: &Path) -> Result<(), String> {
    if !SYSTEMS.contains(&capture.os.as_str()) || capture.run_id == 0 || capture.attempt == 0 {
        return Err("invalid capture identity".to_owned());
    }
    if capture.profile != CANONICAL_PROFILE {
        return Err("capture profile differs from the current runner".to_owned());
    }
    let ids = capture
        .fixtures
        .iter()
        .map(|fixture| fixture.id.clone())
        .collect::<Vec<_>>();
    if ids != fixture_universe(corpus)? {
        return Err(format!(
            "{} fixture inventory differs from the corpus",
            capture.os
        ));
    }
    for fixture in &capture.fixtures {
        let expected_bytes = fs::read(
            corpus
                .join("expectations")
                .join(format!("{}.json", fixture.id)),
        )
        .map_err(|error| format!("{} expectation: {error}", fixture.id))?;
        let expected: Value = serde_json::from_slice(&expected_bytes)
            .map_err(|error| format!("{} expectation: {error}", fixture.id))?;
        let expected_valid = expected
            .get("valid")
            .map(|value| {
                value
                    .as_bool()
                    .ok_or_else(|| format!("{} expectation has invalid status", fixture.id))
            })
            .transpose()?;
        if fixture.valid != expected_valid {
            return Err(format!("{} status differs from the corpus", fixture.id));
        }
        let expected_canonical: &[Value] = match expected.get("canonical") {
            Some(Value::Array(entries)) => entries,
            None => &[],
            Some(_) => {
                return Err(format!(
                    "{} expectation has invalid canonical array",
                    fixture.id
                ))
            }
        };
        if fixture.canonical.len() != expected_canonical.len() {
            return Err(format!(
                "{} canonical key count differs from the corpus",
                fixture.id
            ));
        }
        for (actual, expected) in fixture.canonical.iter().zip(expected_canonical) {
            if actual.identity != require_string(expected, "identity")?
                || actual.kind != require_string(expected, "kind")?
            {
                return Err(format!(
                    "{} canonical key differs from the corpus",
                    fixture.id
                ));
            }
        }
        let expected_diagnostics = expected
            .get("diagnostics")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{} expectation has no diagnostic array", fixture.id))?
            .iter()
            .map(|item| {
                Ok(DiagnosticEntry {
                    code: require_string(item, "code")?.to_owned(),
                    path: require_string(item, "path")?.to_owned(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        if fixture.diagnostics != expected_diagnostics {
            return Err(format!(
                "{} diagnostic sequence differs from the corpus",
                fixture.id
            ));
        }
        if fixture.valid == Some(false) && fixture.diagnostics.is_empty() {
            return Err(format!("{} has an empty rejection", fixture.id));
        }
        if fixture.valid == Some(false) && !fixture.canonical.is_empty() {
            return Err(format!("{} has canonical output on rejection", fixture.id));
        }
        let mut keys = std::collections::BTreeSet::new();
        for entry in &fixture.canonical {
            if !keys.insert((&entry.identity, &entry.kind)) {
                return Err(format!("{} has a duplicate canonical key", fixture.id));
            }
        }
    }
    Ok(())
}

fn compare(
    captures: &[Capture; 3],
    corpus: &Path,
    run_id: u64,
    attempt: u32,
) -> Result<(), String> {
    for capture in captures {
        validate_capture(capture, corpus)?;
    }
    compare_values(captures, run_id, attempt)
}

fn compare_values(captures: &[Capture; 3], run_id: u64, attempt: u32) -> Result<(), String> {
    if run_id == 0 || attempt == 0 {
        return Err("invalid requested run ID or attempt".to_owned());
    }
    let mut by_os = BTreeMap::new();
    for capture in captures {
        if capture.run_id != run_id || capture.attempt != attempt {
            return Err(format!("{} has different run provenance", capture.os));
        }
        if by_os.insert(capture.os.as_str(), capture).is_some() {
            return Err(format!("duplicate {} artifact", capture.os));
        }
    }
    if by_os.keys().copied().collect::<Vec<_>>() != SYSTEMS {
        return Err("missing or extra OS artifact".to_owned());
    }
    let baseline = by_os["linux"];
    for os in ["macos", "windows"] {
        let peer = by_os[os];
        if baseline.fixtures.len() != peer.fixtures.len() {
            return Err(format!("{os}: fixture count changed"));
        }
        for (left, right) in baseline.fixtures.iter().zip(&peer.fixtures) {
            if left.id != right.id || left.valid != right.valid {
                return Err(format!(
                    "{os}: {} fixture identity or status changed",
                    left.id
                ));
            }
            if left.canonical.len() != right.canonical.len() {
                return Err(format!("{os}: {} canonical output count changed", left.id));
            }
            for (expected, actual) in left.canonical.iter().zip(&right.canonical) {
                if expected.identity != actual.identity || expected.kind != actual.kind {
                    return Err(format!("{os}: {} canonical key changed", left.id));
                }
                if expected.bytes.as_bytes() != actual.bytes.as_bytes() {
                    return Err(format!("{os}: {} canonical bytes changed", left.id));
                }
            }
            if left.diagnostics != right.diagnostics {
                return Err(format!("{os}: {} ordered diagnostics changed", left.id));
            }
        }
    }
    Ok(())
}

fn read_capture(path: &Path) -> Result<Capture, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn run(args: &[String]) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let corpus = &root.join("corpus/contract-v0.1");
    let schemas = &root.join("schemas");
    match args {
        [] => self_test(),
        [mode] if mode == "selftest" => self_test(),
        [mode, os, run, attempt, output] if mode == "capture" => {
            let run_id = run.parse::<u64>().map_err(|error| error.to_string())?;
            let attempt = attempt.parse::<u32>().map_err(|error| error.to_string())?;
            let capture = capture(corpus, schemas, run_id, attempt, os)?;
            let bytes = serde_json::to_vec(&capture).map_err(|error| error.to_string())?;
            fs::write(output, bytes).map_err(|error| error.to_string())
        }
        [mode, run, attempt, linux, macos, windows] if mode == "compare" => {
            let run_id = run.parse::<u64>().map_err(|error| error.to_string())?;
            let attempt = attempt.parse::<u32>().map_err(|error| error.to_string())?;
            let captures = [
                read_capture(Path::new(linux))?,
                read_capture(Path::new(macos))?,
                read_capture(Path::new(windows))?,
            ];
            compare(&captures, corpus, run_id, attempt)
        }
        _ => Err("usage: tc019_determinism selftest | capture <linux|macos|windows> <run-id> <attempt> <output> | compare <run-id> <attempt> <linux-file> <macos-file> <windows-file>".to_owned()),
    }
}

fn main() -> ExitCode {
    match run(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("TC-019: {error}");
            ExitCode::FAILURE
        }
    }
}

fn self_test() -> Result<(), String> {
    let other_os = SYSTEMS
        .into_iter()
        .find(|os| *os != env::consts::OS)
        .ok_or_else(|| "no alternate OS in supported systems".to_owned())?;
    let wrong_host = capture(
        Path::new("missing-corpus"),
        Path::new("missing-schemas"),
        42,
        2,
        other_os,
    );
    match wrong_host {
        Err(error) if error.contains("differs from executing host") => {}
        other => return Err(format!("mislabeled host capture accepted: {other:?}")),
    }
    let baseline = FixtureEntry {
        id: "fixture".to_owned(),
        valid: Some(true),
        canonical: vec![CanonicalEntry {
            identity: "id".to_owned(),
            kind: "kind".to_owned(),
            bytes: "one".to_owned(),
        }],
        diagnostics: vec![
            DiagnosticEntry {
                code: "first".to_owned(),
                path: "a".to_owned(),
            },
            DiagnosticEntry {
                code: "second".to_owned(),
                path: "b".to_owned(),
            },
        ],
    };
    let original = SYSTEMS.map(|os| Capture {
        run_id: 42,
        attempt: 2,
        os: os.to_owned(),
        profile: CANONICAL_PROFILE.to_owned(),
        fixtures: vec![baseline.clone()],
    });
    compare_values(&original, 42, 2)?;
    let mut changed = original.clone();
    changed[1].fixtures[0].canonical[0].bytes = "two".to_owned();
    expect_refusal(&changed, "canonical bytes")?;
    let mut changed = original.clone();
    changed[2].fixtures[0].diagnostics.swap(0, 1);
    expect_refusal(&changed, "ordered diagnostics")?;
    let mut changed = original.clone();
    changed[2].fixtures[0].diagnostics.pop();
    expect_refusal(&changed, "ordered diagnostics")?;
    let mut changed = original.clone();
    changed[1].os = "linux".to_owned();
    expect_refusal(&changed, "duplicate")?;
    let mut changed = original.clone();
    changed[2].attempt = 3;
    expect_refusal(&changed, "provenance")?;
    let mut changed = original.clone();
    changed[2].fixtures.clear();
    expect_refusal(&changed, "fixture")?;
    Ok(())
}

fn expect_refusal(captures: &[Capture; 3], reason: &str) -> Result<(), String> {
    match compare_values(captures, 42, 2) {
        Err(error) if error.contains(reason) => Ok(()),
        other => Err(format!("mutation expected {reason} refusal, got {other:?}")),
    }
}
