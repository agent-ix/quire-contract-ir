// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-445's producer-origin assertion using the authoritative QSL executables.
//! Set `QUIRE_SPEC_LANGUAGE_DIR` after building that checkout's `quire-spec`
//! and `config_version_fixtures` binaries from its current source. An absent
//! producer leaves this external case unrun in the ordinary IR test lane.

use crate::support::checked_package::sha256_hex;
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedClauseComparison, CheckedComparisonClauseKind, CheckedComparisonObservation,
    CheckedComparisonOperandIdentity, CheckedComparisonOperandKind, CheckedComparisonOperator,
    CheckedComparisonPath, CheckedComparisonRead, CheckedComparisonRefusal, CheckedNodeId,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
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
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("could not remove {}: {error}", self.0.display());
        }
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

fn selected_model(wire: &Value) -> &str {
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

fn selected_model_variant(
    case: &Path,
    original_unit: &str,
    model: &Value,
    expression: &str,
) -> PathBuf {
    let model_path = case.join("spine-model.semantic-ir.json");
    let digest = quire_canonical::sha256(model, quire_canonical::Limits::new(1 << 20))
        .expect("selected producer model digest")
        .to_string();
    let selected = original_unit
        .split("digest \"sha256-jcs:")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("producer model selection");
    let unit = original_unit.replacen(selected, &digest, 1).replacen(
        "self.versionNumber = pre(self.versionNumber)",
        expression,
        1,
    );
    std::fs::write(
        model_path,
        serde_json::to_vec(model).expect("selected model JSON"),
    )
    .expect("write selected model");
    std::fs::write(case.join("spine-unit.native"), unit).expect("write selected-model variant");
    compile_request(case)
}

fn bounded_input_variant(case: &Path, original_unit: &str, original_model: &Value) -> PathBuf {
    let mut model = original_model.clone();
    let parameter = {
        let mut field = model["types"][1]["fields"][0].clone();
        let identity = "ix://example/config-version/ConfigVersion/attemptUpdate/delta";
        field["identity"] = json!(identity);
        field["name"] = json!("delta");
        field["origin"]["generated"]["generatorIdentity"] = json!(identity);
        field["origin"]["generated"]["inputIdentities"] = json!([identity]);
        field
    };
    model["types"][1]["operations"][0]["params"] = json!([parameter]);
    selected_model_variant(case, original_unit, &model, "self.versionNumber = delta")
}

fn admit_selected_variant(case: &Path, bytes: &[u8]) -> CheckedPackageV2 {
    let wire: Value = serde_json::from_slice(bytes).expect("QSL variant V2 JSON");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(
            selected_model(&wire),
            std::fs::read(case.join("spine-model.semantic-ir.json"))
                .expect("variant selected model"),
        )
        .expect("domain evidence declarations agree on exact bytes");
    evidence.support_feature("quire.value.complete/v1");
    match CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("QSL selected-model variant must admit: {other:?}"),
    }
}

fn version_unchanged(package: &CheckedPackageV2) -> CheckedClauseComparison {
    let postconditions: Vec<_> = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "state_clause"
        })
        .flat_map(|clause| {
            clause
                .occurrences
                .iter()
                .filter(|occurrence| {
                    occurrence.role == quire_contract_model::CheckedOccurrenceRole::Claim
                })
                .filter_map(|claim| {
                    package
                        .direct_integer_comparison(&clause.node_id, claim)
                        .ok()
                })
        })
        .filter(|projection| projection.kind == CheckedComparisonClauseKind::Postcondition)
        .collect();
    let [postcondition] = postconditions.as_slice() else {
        panic!("producer emits exactly one eligible postcondition, got {postconditions:?}");
    };
    postcondition.clone()
}

fn external_consumer_matches_closed_types(projection: &CheckedClauseComparison) {
    match projection.operator {
        CheckedComparisonOperator::Eq
        | CheckedComparisonOperator::Ne
        | CheckedComparisonOperator::Lt
        | CheckedComparisonOperator::Le
        | CheckedComparisonOperator::Gt
        | CheckedComparisonOperator::Ge => {}
    }
    for operand in &projection.operands {
        match &operand.identity {
            CheckedComparisonOperandIdentity::GraphChild(_) => {}
            CheckedComparisonOperandIdentity::InlineLiteral { .. } => {}
        }
        match &operand.kind {
            CheckedComparisonOperandKind::Literal => {}
            CheckedComparisonOperandKind::Read {
                provenance,
                observation,
            } => {
                match provenance {
                    CheckedComparisonRead::StateField { .. }
                    | CheckedComparisonRead::OperationInput { .. } => {}
                }
                match observation {
                    CheckedComparisonObservation::Current
                    | CheckedComparisonObservation::Pre
                    | CheckedComparisonObservation::Post => {}
                }
            }
        }
    }
}

fn external_consumer_matches_closed_refusal(error: &quire_contract_model::CheckedComparisonError) {
    match error.kind {
        CheckedComparisonRefusal::UnknownClause
        | CheckedComparisonRefusal::NotStateClause
        | CheckedComparisonRefusal::MissingClaimOccurrence
        | CheckedComparisonRefusal::UnsupportedClauseKind
        | CheckedComparisonRefusal::MissingComparison
        | CheckedComparisonRefusal::UnknownOperator
        | CheckedComparisonRefusal::IneligibleOperator
        | CheckedComparisonRefusal::InvalidComparisonShape
        | CheckedComparisonRefusal::IneligibleOperand
        | CheckedComparisonRefusal::MissingChild
        | CheckedComparisonRefusal::NoRead
        | CheckedComparisonRefusal::MissingRange
        | CheckedComparisonRefusal::UnboundedRange
        | CheckedComparisonRefusal::RangeOutOfI128 => {}
    }
    match error.path {
        CheckedComparisonPath::Clause
        | CheckedComparisonPath::Condition
        | CheckedComparisonPath::Comparison
        | CheckedComparisonPath::Operand(_) => {}
    }
}

/// Trace: FR-038-AC-202, FR-038-AC-203, FR-038-AC-204, FR-038-AC-205, FR-038-AC-206, FR-038-AC-208, FR-038-AC-209
#[trace(
    "FR-038-AC-202",
    "FR-038-AC-203",
    "FR-038-AC-204",
    "FR-038-AC-205",
    "FR-038-AC-206",
    "FR-038-AC-208",
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
    let scratch = Scratch::new();
    run(&fixture, &[&scratch.0]);
    let case = scratch.0.join("unchanged-version");
    let original_unit =
        std::fs::read_to_string(case.join("spine-unit.native")).expect("fresh QSL spine source");
    assert!(original_unit.contains("self.versionNumber = pre(self.versionNumber)"));
    let request = compile_request(&case);
    eprintln!(
        "TC-445 QSL revision={} commands='{} {}; {} compile {}'",
        String::from_utf8_lossy(&revision.stdout).trim(),
        fixture.display(),
        scratch.0.display(),
        producer.display(),
        request.display()
    );
    let bytes = run(&producer, &[Path::new("compile"), &request]);
    let wire: Value = serde_json::from_slice(&bytes).expect("QSL emits V2 JSON");
    let model = std::fs::read(
        scratch
            .0
            .join("unchanged-version/spine-model.semantic-ir.json"),
    )
    .expect("selected QSL model document");
    let original_model: Value = serde_json::from_slice(&model).expect("producer model JSON");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(selected_model(&wire), model)
        .expect("domain evidence declarations agree on exact bytes");
    evidence.support_feature("quire.value.complete/v1");
    let package =
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("fresh QSL package must be admitted by IR: {other:?}"),
        };
    let before = package.clone();
    let result = version_unchanged(&package);
    external_consumer_matches_closed_types(&result);
    let missing_clause: CheckedNodeId = serde_json::from_value(json!({
        "domain": "quire.checked-semantic-node/v1", "digest": "f".repeat(64)
    }))
    .expect("well-shaped unknown clause id");
    let refusal = package
        .direct_integer_comparison(&missing_clause, &result.claim)
        .expect_err("unknown clause");
    external_consumer_matches_closed_refusal(&refusal);
    assert_eq!(refusal.kind, CheckedComparisonRefusal::UnknownClause);
    assert_eq!(result.operator, CheckedComparisonOperator::Eq);
    assert_eq!(result.operands[0].ordinal, 0);
    assert_eq!(result.operands[1].ordinal, 1);
    let read = |index: usize| match &result.operands[index].kind {
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

    let contextual = format!(
        "{original_unit}\ninvariant VersionFloor using v on Config::ConfigVersion at current \
         {{ self.versionNumber >= 0 }}\npre VersionReady using v on \
         Config::ConfigVersion::attemptUpdate {{ self.versionNumber >= 0 }}\n"
    );
    std::fs::write(case.join("spine-unit.native"), contextual)
        .expect("write current-observation clauses");
    let contextual_request = compile_request(&case);
    let contextual_bytes = run(&producer, &[Path::new("compile"), &contextual_request]);
    let contextual_package = match CheckedPackageV2::read(
        &contextual_bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("QSL current-observation clauses must admit: {other:?}"),
    };
    for kind in [
        CheckedComparisonClauseKind::Invariant,
        CheckedComparisonClauseKind::Precondition,
    ] {
        let projections: Vec<_> = contextual_package
            .graph()
            .nodes
            .iter()
            .filter(|node| {
                node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "state_clause"
            })
            .flat_map(|clause| {
                clause.occurrences.iter().filter_map(|claim| {
                    contextual_package
                        .direct_integer_comparison(&clause.node_id, claim)
                        .ok()
                })
            })
            .filter(|projection| projection.kind == kind)
            .collect();
        let [projection] = projections.as_slice() else {
            panic!("one eligible {kind:?} comparison, got {projections:?}");
        };
        assert_eq!(projection.operator, CheckedComparisonOperator::Ge);
        assert_eq!(projection.operands[0].ordinal, 0);
        assert_eq!(projection.operands[1].ordinal, 1);
        assert!(matches!(
            &projection.operands[0].kind,
            CheckedComparisonOperandKind::Read {
                provenance: CheckedComparisonRead::StateField { field, .. },
                observation: CheckedComparisonObservation::Current,
            } if field.as_ref() == "versionNumber"
        ));
        assert_eq!(
            (
                projection.operands[0].range.lower,
                projection.operands[0].range.upper
            ),
            (0, 1000)
        );
        assert!(matches!(
            projection.operands[1].kind,
            CheckedComparisonOperandKind::Literal
        ));
        assert_eq!(
            (
                projection.operands[1].range.lower,
                projection.operands[1].range.upper
            ),
            (0, 0)
        );
        let CheckedComparisonOperandIdentity::GraphChild(literal_id) =
            &projection.operands[1].identity
        else {
            panic!(
                "producer literal is a graph child: {:?}",
                projection.operands[1].identity
            );
        };
        assert!(contextual_package.graph().nodes.iter().any(|node| {
            &node.node_id == literal_id
                && node.node_tag.as_ref() == "value"
                && node.semantic_form.as_ref() == "literal"
        }));
    }

    let reversed = original_unit.replacen(
        "self.versionNumber = pre(self.versionNumber)",
        "pre(self.versionNumber) = self.versionNumber",
        1,
    );
    std::fs::write(case.join("spine-unit.native"), reversed).expect("write reversed reads");
    let reversed_request = compile_request(&case);
    let reversed_bytes = run(&producer, &[Path::new("compile"), &reversed_request]);
    let reversed_package = match CheckedPackageV2::read(
        &reversed_bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("QSL reversed read pair must admit: {other:?}"),
    };
    let reversed = version_unchanged(&reversed_package);
    for (ordinal, source) in [(0, 1), (1, 0)] {
        assert_eq!(reversed.operands[ordinal].ordinal, ordinal as u64);
        assert_eq!(
            reversed.operands[ordinal].identity,
            result.operands[source].identity
        );
        assert_eq!(
            reversed.operands[ordinal].kind,
            result.operands[source].kind
        );
        assert_eq!(
            reversed.operands[ordinal].range,
            result.operands[source].range
        );
    }
    assert_ne!(reversed.operands[0].identity, reversed.operands[1].identity);

    let repeated = format!(
        "{original_unit}\npost VersionUnchangedAgain using v on Config::ConfigVersion::attemptUpdate \
         {{ self.versionNumber = pre(self.versionNumber) }}\n"
    );
    std::fs::write(case.join("spine-unit.native"), repeated).expect("write repeated clause");
    let repeated_request = compile_request(&case);
    let repeated_bytes = run(&producer, &[Path::new("compile"), &repeated_request]);
    let repeated_package = match CheckedPackageV2::read(
        &repeated_bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("QSL repeated clause must admit: {other:?}"),
    };
    let repeated_clause = repeated_package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "state"
                && node.semantic_form.as_ref() == "state_clause"
                && node
                    .occurrences
                    .iter()
                    .filter(|occurrence| {
                        occurrence.role == quire_contract_model::CheckedOccurrenceRole::Claim
                    })
                    .count()
                    >= 2
        })
        .expect("two producer claims share one content-addressed clause");
    let claims: Vec<_> = repeated_clause
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.role == quire_contract_model::CheckedOccurrenceRole::Claim)
        .take(2)
        .collect();
    let first = repeated_package
        .direct_integer_comparison(&repeated_clause.node_id, claims[0])
        .expect("first authentic claim");
    let second = repeated_package
        .direct_integer_comparison(&repeated_clause.node_id, claims[1])
        .expect("second authentic claim");
    assert_eq!(first.comparison, second.comparison);
    assert_eq!(first.operands, second.operands);
    assert_ne!(first.claim, second.claim);

    let mut extreme_model = original_model.clone();
    extreme_model["types"][0]["constraints"][0]["operands"]["value"] = json!(i128::MIN.to_string());
    extreme_model["types"][0]["constraints"][1]["operands"]["value"] = json!(i128::MAX.to_string());
    let extreme_request = selected_model_variant(
        &case,
        &original_unit,
        &extreme_model,
        "self.versionNumber = pre(self.versionNumber)",
    );
    let extreme_bytes = run(&producer, &[Path::new("compile"), &extreme_request]);
    let extreme_package = admit_selected_variant(&case, &extreme_bytes);
    let extreme = version_unchanged(&extreme_package);
    for operand in &extreme.operands {
        assert!(matches!(
            operand.kind,
            CheckedComparisonOperandKind::Read { .. }
        ));
        assert_eq!(
            (operand.range.lower, operand.range.upper),
            (i128::MIN, i128::MAX)
        );
    }

    let mut unbounded_model = original_model.clone();
    unbounded_model["types"][1]["fields"][0]["typeRef"] = json!("ix://quire/native/Integer");
    let unbounded_request = selected_model_variant(
        &case,
        &original_unit,
        &unbounded_model,
        "self.versionNumber = pre(self.versionNumber)",
    );
    let unbounded_bytes = run(&producer, &[Path::new("compile"), &unbounded_request]);
    let unbounded_package = admit_selected_variant(&case, &unbounded_bytes);
    let refusals: Vec<_> = unbounded_package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "state_clause"
        })
        .flat_map(|clause| {
            clause.occurrences.iter().filter_map(|claim| {
                unbounded_package
                    .direct_integer_comparison(&clause.node_id, claim)
                    .err()
            })
        })
        .filter(|error| error.kind == CheckedComparisonRefusal::UnboundedRange)
        .collect();
    let [unbounded] = refusals.as_slice() else {
        panic!("one unbounded selected field refusal, got {refusals:?}");
    };
    assert_eq!(unbounded.path, CheckedComparisonPath::Operand(0));
    assert!(unbounded.child.is_some());

    // The fresh producer's selected model is extended with a bounded input.
    // Recompute its selection digest and compile the changed source through QSL.
    let input_request = bounded_input_variant(&case, &original_unit, &original_model);
    let input_bytes = run(&producer, &[Path::new("compile"), &input_request]);
    let input_package = admit_selected_variant(&case, &input_bytes);
    let input = version_unchanged(&input_package);
    assert!(matches!(
        &input.operands[1].kind,
        CheckedComparisonOperandKind::Read {
            provenance: CheckedComparisonRead::OperationInput { ordinal: 0, .. },
            observation: CheckedComparisonObservation::Current,
        }
    ));
    assert_eq!(
        (input.operands[1].range.lower, input.operands[1].range.upper),
        (0, 1000)
    );
}
