//! FR-018-AC-2: each mutation kind is reported on its own.
//!
//! The combined test `tc_018_all_mismatch_kinds_and_exit_classes_are_stable`
//! (in `conformance.rs`) applies several mutations to one fixture at once, so it
//! cannot show that a kind is detected independently. Here every case copies
//! the corpus afresh, applies exactly one mutation to one expectation, and
//! asserts that the mutated fixture reports exactly that one kind, every other
//! fixture still matches, and the exit class is the stable mismatch class (1).

use std::{collections::BTreeSet, path::PathBuf, process::Output};

use ix_trace_rs::trace;
use serde_json::{json, Value};

use crate::support::corpus::{read_json, rows, write_json, Scratch};

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

fn expectation_path(scratch: &Scratch, fixture: &str) -> PathBuf {
    scratch.0.join(format!("expectations/{fixture}.json"))
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
    let mut seen = false;
    for row in &rows(output) {
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

/// Applies exactly the named case's one mutation to a fresh corpus copy.
fn run_case(label: &str) {
    let case = CASES.iter().find(|case| case.label == label).unwrap();
    let scratch = Scratch::corpus(&format!("ac2-{label}"));
    let path = expectation_path(&scratch, case.fixture);
    let mut expectation = read_json(&path);
    let before = expectation.clone();
    (case.mutate)(&mut expectation);
    assert_ne!(before, expectation, "{label}: mutation changed nothing");
    write_json(&path, &expectation);
    assert_only(&scratch.run(), case.fixture, case.kind, label);
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_validity_alone_reports_only_its_kind() {
    run_case("validity");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_diagnostic_code_alone_reports_only_its_kind() {
    run_case("diagnostic-code");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_diagnostic_path_alone_reports_only_its_kind() {
    run_case("diagnostic-path");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_diagnostic_span_alone_reports_only_its_kind() {
    run_case("diagnostic-span");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_diagnostic_obligation_alone_reports_only_its_kind() {
    run_case("diagnostic-obligation");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_canonical_bytes_alone_reports_only_its_kind() {
    run_case("canonical-bytes");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_dependencies_alone_reports_only_its_kind() {
    run_case("dependencies");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_coverage_row_alone_reports_only_its_kind() {
    run_case("coverage-row");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_mismatch_coverage_reason_alone_reports_only_its_kind() {
    run_case("coverage-reason");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_the_case_table_produces_every_fr020_mismatch_kind() {
    // FR-020 fixes five kinds. The runner exposes no public kind list, so this
    // hand-typed set cannot notice a sixth runner kind; it only guards the
    // table above against silently losing one of the five.
    let produced: BTreeSet<_> = CASES.iter().map(|case| case.kind).collect();
    assert_eq!(
        produced.into_iter().collect::<Vec<_>>(),
        [
            "canonical_bytes",
            "coverage",
            "dependencies",
            "diagnostics",
            "validity"
        ]
    );
}

/// The two-diagnostic baseline. No shipped fixture yields two diagnostics, so
/// the stale-revision coverage input gets a second stale trace (setup, not the
/// mutation) and the observed result becomes the expectation. Returns the
/// scratch copy, the expectation path, the expectation and its diagnostics.
fn two_diagnostic_baseline(label: &str) -> (Scratch, PathBuf, Value, Vec<Value>) {
    let fixture = "coverage-stale";
    let scratch = Scratch::corpus(&format!("ac2-{label}"));
    let input_path = scratch.0.join(format!("inputs/{fixture}.json"));
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

    let path = expectation_path(&scratch, fixture);
    let mut expectation = read_json(&path);
    let observed = rows(&scratch.run())
        .into_iter()
        .find(|row| row["fixture_id"] == fixture)
        .unwrap()["actual"]
        .clone();
    let diagnostics = observed["diagnostics"].as_array().unwrap().clone();
    assert!(diagnostics.len() >= 2, "setup must yield two diagnostics");
    assert_ne!(diagnostics[0], diagnostics[1], "diagnostics must differ");
    expectation["diagnostics"] = json!(diagnostics);
    expectation["coverage"] = observed["coverage"].clone();
    write_json(&path, &expectation);
    assert_eq!(scratch.run().status.code(), Some(0), "{label}: baseline");
    (scratch, path, expectation, diagnostics)
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_diagnostic_order_alone_is_reported_as_a_diagnostics_mismatch() {
    let (scratch, path, mut expectation, mut diagnostics) = two_diagnostic_baseline("order");
    diagnostics.swap(0, 1);
    expectation["diagnostics"] = json!(diagnostics);
    write_json(&path, &expectation);
    assert_only(&scratch.run(), "coverage-stale", "diagnostics", "order");
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_a_drift_in_the_second_diagnostic_alone_is_a_diagnostics_mismatch() {
    let (scratch, path, mut expectation, _) = two_diagnostic_baseline("second-code");
    expectation["diagnostics"][1]["code"] = json!("invalid_identifier");
    write_json(&path, &expectation);
    assert_only(
        &scratch.run(),
        "coverage-stale",
        "diagnostics",
        "second-code",
    );
}

#[trace("TC-018", "FR-018-AC-2")]
#[test]
fn tc_018_a_missing_second_diagnostic_alone_is_a_diagnostics_mismatch() {
    let (scratch, path, mut expectation, mut diagnostics) =
        two_diagnostic_baseline("second-dropped");
    diagnostics.truncate(1);
    expectation["diagnostics"] = json!(diagnostics);
    write_json(&path, &expectation);
    assert_only(
        &scratch.run(),
        "coverage-stale",
        "diagnostics",
        "second-dropped",
    );
}
