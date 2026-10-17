//! FR-018-AC-2: each mutation kind is reported on its own.
//!
//! The combined test `tc_018_all_mismatch_kinds_and_exit_classes_are_stable`
//! (in `conformance.rs`) applies several mutations to one fixture at once, so it
//! cannot show that a kind is detected independently. Here every case copies
//! the corpus afresh, applies exactly one mutation to one expectation, and
//! asserts that the mutated fixture reports exactly that one kind, every other
//! fixture still matches, and the exit class is the stable mismatch class (1).

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use ix_trace_rs::trace;
use serde_json::{json, Value};

const ZERO_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// One mutation of one expectation: where it applies, how, and the single
/// mismatch kind it must produce.
struct Case {
    label: &'static str,
    fixture: &'static str,
    mutate: fn(&mut Value),
    kind: &'static str,
}

const CASES: &[Case] = &[
    Case {
        label: "validity",
        fixture: "package-constructs",
        mutate: |e| e["valid"] = json!(false),
        kind: "validity",
    },
    Case {
        label: "diagnostic-code",
        fixture: "package-duplicate-clause",
        mutate: |e| e["diagnostics"][0]["code"] = json!("invalid_identifier"),
        kind: "diagnostics",
    },
    Case {
        label: "diagnostic-path",
        fixture: "package-duplicate-clause",
        mutate: |e| e["diagnostics"][0]["path"] = json!("mutated"),
        kind: "diagnostics",
    },
    Case {
        label: "diagnostic-span",
        fixture: "package-duplicate-clause",
        mutate: |e| e["diagnostics"][0]["span"]["end"]["column"] = json!(9),
        kind: "diagnostics",
    },
    Case {
        label: "diagnostic-obligation",
        fixture: "expression-obligation-index-in-bounds",
        mutate: |e| e["diagnostics"][0]["obligation_kind"] = json!("non_zero_divisor"),
        kind: "diagnostics",
    },
    Case {
        label: "canonical-bytes",
        fixture: "package-constructs",
        mutate: |e| e["canonical"][0]["bytes_path"] = json!("canonical/package-constructs-1.json"),
        kind: "canonical_bytes",
    },
    Case {
        label: "canonical-digest",
        fixture: "package-constructs",
        mutate: |e| e["canonical"][0]["digest"] = json!(ZERO_DIGEST),
        kind: "canonical_digest",
    },
    Case {
        label: "dependencies",
        fixture: "package-reference",
        mutate: |e| e["dependencies"][0]["path"] = json!(["mutated"]),
        kind: "dependencies",
    },
    Case {
        label: "coverage-row",
        fixture: "coverage-shallow",
        mutate: |e| e["coverage"]["requirements"][1]["class"] = json!("uncovered"),
        kind: "coverage",
    },
    Case {
        label: "coverage-reason",
        fixture: "coverage-stale",
        mutate: |e| e["coverage"]["artifacts"][0]["orphan_reason"] = json!("duplicate_artifact"),
        kind: "coverage",
    },
];

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn runner(corpus: &Path, schemas: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
        .args(["run", "--corpus"])
        .arg(corpus)
        .arg("--schemas")
        .arg(schemas)
        .output()
        .unwrap()
}

/// A fresh copy of the corpus and schemas, removed on drop.
struct Fresh {
    root: PathBuf,
}

impl Fresh {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "quire-contract-ir-fr018ac2-{}-{label}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        copy_tree(
            &repository().join("corpus/contract-v0.1"),
            &root.join("corpus"),
        );
        copy_tree(&repository().join("schemas"), &root.join("schemas"));
        Self { root }
    }

    fn corpus(&self) -> PathBuf {
        self.root.join("corpus")
    }

    fn expectation(&self, fixture: &str) -> PathBuf {
        self.corpus().join(format!("expectations/{fixture}.json"))
    }

    fn input(&self, fixture: &str) -> PathBuf {
        self.corpus().join(format!("inputs/{fixture}.json"))
    }

    fn run(&self) -> Output {
        runner(&self.corpus(), &self.root.join("schemas"))
    }
}

impl Drop for Fresh {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn copy_tree(source: &Path, target: &Path) {
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

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn rows(output: &Output) -> Vec<Value> {
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| serde_json::from_slice::<Value>(row).unwrap())
        .collect()
}

fn kinds(row: &Value) -> Vec<&str> {
    row["mismatch_kinds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|kind| kind.as_str().unwrap())
        .collect()
}

/// Asserts a run produced the stable mismatch class and that exactly the one
/// `fixture` row reports exactly `[kind]`; every other row matches with no kinds.
fn assert_only(output: &Output, fixture: &str, kind: &str, label: &str) {
    assert_eq!(output.status.code(), Some(1), "{label}: exit class");
    assert!(output.stderr.is_empty(), "{label}: stderr");
    let rows = rows(output);
    let mut seen = false;
    for row in &rows {
        if row["fixture_id"] == fixture {
            seen = true;
            assert_eq!(row["status"], "mismatch", "{label}: status");
            assert_eq!(kinds(row), [kind], "{label}: kinds");
        } else {
            assert_eq!(row["status"], "match", "{label}: {}", row["fixture_id"]);
            assert!(kinds(row).is_empty(), "{label}: {}", row["fixture_id"]);
        }
    }
    assert!(seen, "{label}: fixture {fixture} has no row");
}

/// Trace: TC-018, FR-018-AC-2
#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_each_mismatch_kind_is_reported_independently_for_one_mutation() {
    let baseline = Fresh::new("baseline").run();
    assert_eq!(baseline.status.code(), Some(0));
    assert!(baseline.stderr.is_empty());

    for case in CASES {
        let fresh = Fresh::new(case.label);
        let path = fresh.expectation(case.fixture);
        let mut expectation = read_json(&path);
        let before = expectation.clone();
        (case.mutate)(&mut expectation);
        assert_ne!(
            before, expectation,
            "{}: mutation changed nothing",
            case.label
        );
        write_json(&path, &expectation);
        assert_only(&fresh.run(), case.fixture, case.kind, case.label);
    }

    // Every kind the criterion names is produced by some case above, so the
    // table cannot silently lose one.
    let produced: std::collections::BTreeSet<_> = CASES.iter().map(|case| case.kind).collect();
    assert_eq!(
        produced.into_iter().collect::<Vec<_>>(),
        [
            "canonical_bytes",
            "canonical_digest",
            "coverage",
            "dependencies",
            "diagnostics",
            "validity"
        ]
    );
}

/// Trace: TC-018, FR-018-AC-2
#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_diagnostic_order_alone_is_reported_as_a_diagnostics_mismatch() {
    // No shipped fixture yields two diagnostics, so give the stale-revision
    // coverage input a second stale trace (setup, not the mutation) and take the
    // observed result as the expectation. The single mutation is then swapping
    // the two diagnostics.
    let fresh = Fresh::new("diagnostic-order");
    let fixture = "coverage-stale";
    let input_path = fresh.input(fixture);
    let mut input = read_json(&input_path);
    let traces = input["traces"].as_array_mut().unwrap();
    let mut second = traces[0].clone();
    second["artifact_id"] = json!("stale-second");
    second["target_span"]["start"]["column"] = json!(20);
    second["target_span"]["start"]["byte_offset"] = json!(19);
    second["target_span"]["end"]["column"] = json!(21);
    second["target_span"]["end"]["byte_offset"] = json!(20);
    traces.push(second);
    write_json(&input_path, &input);

    let path = fresh.expectation(fixture);
    let mut expectation = read_json(&path);
    let observed = rows(&fresh.run())
        .into_iter()
        .find(|row| row["fixture_id"] == fixture)
        .unwrap()["actual"]
        .clone();
    let observed_diagnostics = observed["diagnostics"].as_array().unwrap().clone();
    assert!(
        observed_diagnostics.len() >= 2,
        "setup must yield two diagnostics"
    );
    assert_ne!(
        observed_diagnostics[0], observed_diagnostics[1],
        "swap must change the order"
    );
    expectation["diagnostics"] = json!(observed_diagnostics);
    expectation["coverage"] = observed["coverage"].clone();
    write_json(&path, &expectation);
    let matched = fresh.run();
    assert_eq!(matched.status.code(), Some(0));

    let mut swapped = observed_diagnostics;
    swapped.swap(0, 1);
    expectation["diagnostics"] = json!(swapped);
    write_json(&path, &expectation);
    assert_only(&fresh.run(), fixture, "diagnostics", "diagnostic-order");
}
