// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-445's producer-origin assertion using the authoritative QSL executables.
//! Set `QUIRE_SPEC_LANGUAGE_DIR` after building that checkout's `quire-spec`
//! and `config_version_fixtures` binaries from its current source. An absent
//! producer leaves this external case unrun in the ordinary IR test lane.

use crate::support::checked_package::sha256_hex;
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedClauseComparison, CheckedComparisonObservation, CheckedComparisonOperandKind,
    CheckedComparisonOperator, CheckedComparisonRead, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("ir703-qsl-config-version-{}", std::process::id()));
        std::fs::create_dir(&path).expect("unique producer output directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove own producer output");
    }
}

fn run(program: &Path, arguments: &[&Path]) -> Vec<u8> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("start {}: {error}", program.display()));
    assert!(
        output.status.success(),
        "{} failed: {}",
        program.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn selected_model<'a>(wire: &'a Value) -> &'a str {
    wire.pointer("/lock/model_selections/0/digest")
        .and_then(Value::as_str)
        .expect("ConfigVersion selects its domain model")
}

fn compile_request(directory: &Path) -> PathBuf {
    let program = std::fs::read(directory.join("spine-unit.native")).expect("fresh spine unit");
    let model = std::fs::read(directory.join("spine-model.semantic-ir.json"))
        .expect("fresh semantic model");
    let source = |file: &str, identity: &str, document: &str, bytes: &[u8]| {
        json!({
            "file": file,
            "authority": "agent-ix",
            "identity": identity,
            "revision_namespace": "example",
            "revision": "1",
            "digest": format!("sha256:{}", sha256_hex(bytes)),
            "document": document,
            "formal_revision": 1,
        })
    };
    let request = json!({"format": "native-compile/1", "request": {
        "models": [{"format": "semantic-ir/2.0.0", "source": source(
            "spine-model.semantic-ir.json", "ix://example/config-version/spine/model",
            "ConfigVersionSpineModel", &model)}],
        "program": {"source": source("spine-unit.native",
            "ix://example/config-version/spine/unit", "ConfigVersionSpineUnit", &program)},
    }});
    let path = directory.join("ir703-compile.json");
    std::fs::write(&path, serde_json::to_vec(&request).expect("request JSON"))
        .expect("write own producer request");
    path
}

fn version_unchanged(package: &CheckedPackageV2) -> CheckedClauseComparison {
    let clause = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "state"
                && node.semantic_form.as_ref() == "state_clause"
                && node.declaration.as_ref().is_some_and(|declaration| {
                    declaration
                        .qualified_name
                        .last()
                        .is_some_and(|name| name.as_ref() == "VersionUnchanged")
                })
        })
        .expect("producer's VersionUnchanged state clause");
    let claim = clause
        .occurrences
        .iter()
        .find(|occurrence| occurrence.role == quire_contract_model::CheckedOccurrenceRole::Claim)
        .expect("authentic claim occurrence");
    package
        .direct_integer_comparison(&clause.node_id, claim)
        .expect("typed direct comparison")
}

/// Trace: FR-038-AC-202, FR-038-AC-203, FR-038-AC-204, FR-038-AC-205, FR-038-AC-206, FR-038-AC-209
#[trace(
    "FR-038-AC-202",
    "FR-038-AC-203",
    "FR-038-AC-204",
    "FR-038-AC-205",
    "FR-038-AC-206",
    "FR-038-AC-209"
)]
#[test]
fn tc_445_fresh_qsl_config_version_has_shared_field_with_post_and_pre_observations() {
    if std::env::var_os("IR703_QSL_CONFORMANCE").is_none() {
        eprintln!("TC-445 producer-origin case unrun outside the named conformance gate");
        return;
    }
    let Some(root) = std::env::var_os("QUIRE_SPEC_LANGUAGE_DIR").map(PathBuf::from) else {
        panic!("TC-445 conformance requires QUIRE_SPEC_LANGUAGE_DIR");
    };
    let fixture = root.join("target/debug/examples/config_version_fixtures");
    let producer = root.join("target/debug/quire-spec");
    let revision = Command::new("git")
        .args([
            "-C",
            root.to_str().expect("UTF-8 checkout path"),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("inspect producing QSL revision");
    assert!(revision.status.success(), "QSL revision must be available");
    eprintln!(
        "TC-445 QSL revision={} commands='{} <scratch>; {} compile <scratch>/unchanged-version/compile.json'",
        String::from_utf8_lossy(&revision.stdout).trim(),
        fixture.display(), producer.display()
    );
    let scratch = Scratch::new();
    run(&fixture, &[&scratch.0]);
    let case = scratch.0.join("unchanged-version");
    let original_unit =
        std::fs::read_to_string(case.join("spine-unit.native")).expect("fresh QSL spine source");
    assert!(original_unit.contains("self.versionNumber = pre(self.versionNumber)"));
    let request = compile_request(&case);
    let bytes = run(&producer, &[Path::new("compile"), &request]);
    let wire: Value = serde_json::from_slice(&bytes).expect("QSL emits V2 JSON");
    let model = std::fs::read(
        scratch
            .0
            .join("unchanged-version/spine-model.semantic-ir.json"),
    )
    .expect("selected QSL model document");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.insert_domain_package_document(selected_model(&wire), model);
    evidence.support_feature("quire.value.complete/v1");
    let package =
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("fresh QSL package must be admitted by IR: {other:?}"),
        };
    let before = package.clone();
    let result = version_unchanged(&package);
    assert_eq!(result.operator, CheckedComparisonOperator::Eq);
    assert_eq!(result.operands[0].ordinal, 0);
    assert_eq!(result.operands[1].ordinal, 1);
    let read = |index| match &result.operands[index].kind {
        CheckedComparisonOperandKind::Read {
            provenance:
                CheckedComparisonRead::StateField {
                    project,
                    declaration,
                    field,
                },
            observation,
        } => {
            assert_eq!(field.as_ref(), "versionNumber");
            assert!(package
                .graph()
                .nodes
                .iter()
                .any(|node| &node.node_id == declaration));
            (project, *observation)
        }
        other => panic!("expected StateField read, got {other:?}"),
    };
    let (left, left_observation) = read(0);
    let (right, right_observation) = read(1);
    assert_eq!(left, right);
    assert_eq!(left_observation, CheckedComparisonObservation::Post);
    assert_eq!(right_observation, CheckedComparisonObservation::Pre);
    for operand in &result.operands {
        assert_eq!((operand.range.lower, operand.range.upper), (0, 1000));
    }
    assert_eq!(result, version_unchanged(&package));
    assert_eq!(result, version_unchanged(&before));
    assert_eq!(package, before);

    for (spelling, expected) in [
        ("!=", CheckedComparisonOperator::Ne),
        ("<", CheckedComparisonOperator::Lt),
        ("<=", CheckedComparisonOperator::Le),
        (">", CheckedComparisonOperator::Gt),
        (">=", CheckedComparisonOperator::Ge),
    ] {
        let variant = original_unit.replacen(
            "self.versionNumber = pre(self.versionNumber)",
            &format!("self.versionNumber {spelling} pre(self.versionNumber)"),
            1,
        );
        std::fs::write(case.join("spine-unit.native"), variant).expect("write QSL source variant");
        let variant_request = compile_request(&case);
        let bytes = run(&producer, &[Path::new("compile"), &variant_request]);
        let admitted =
            match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
                CheckedPackageV2ReadResult::Admitted(package) => package,
                other => panic!("QSL {spelling} comparison must admit: {other:?}"),
            };
        let projected = version_unchanged(&admitted);
        assert_eq!(projected.operator, expected);
        assert_eq!(projected.operands[0].ordinal, 0);
        assert_eq!(projected.operands[1].ordinal, 1);
    }
}
