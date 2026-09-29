use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    panic::catch_unwind,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use quire_contract_ir::{
    expected_inventory, ContractPackage, DeclarationEnvironment, DiagnosticCode, PackageId,
    RequirementId, RequirementRef, RequirementRevision, SourceDocumentId, SourceIdentity,
    SourceLocation, SourceRevision, SourceSpan, SymbolName, ValidationOptions, ValueDeclaration,
    ValueDeclarationKind, ValueType, MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH,
    MAX_WIRE_JSON_DEPTH,
};
use serde_json::{json, Value};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn corpus() -> PathBuf {
    repository().join("corpus/contract-v0.1")
}

fn runner(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
        .args(arguments)
        .output()
        .unwrap()
}

fn run_corpus(path: &Path) -> Output {
    runner(&["run", "--corpus", path.to_str().unwrap()])
}

struct Scratch(PathBuf);

impl Scratch {
    fn corpus(label: &str) -> Self {
        let path = std::env::temp_dir()
            .join(format!(
                "quire-contract-ir-tc018-{}-{label}",
                std::process::id()
            ))
            .join("contract-v0.1");
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        copy_tree(&corpus(), &path);
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent() {
            let _ = fs::remove_dir_all(parent);
        }
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

fn error_code(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["protocol"], "quire.contract.conformance-jsonl/v1");
    error["code"].as_str().unwrap().to_owned()
}

fn input_count(corpus: &Path) -> usize {
    fs::read_dir(corpus.join("inputs")).unwrap().count()
}

/// Tracing: TC-018, FR-018-AC-1, FR-019-AC-1, FR-019-AC-3, FR-020-AC-1.
/// TC-019.
/// FR-018-AC-1.
/// FR-019-AC-1.
/// FR-019-AC-3.
/// FR-020-AC-1.
#[test]
fn tc_018_the_corpus_runs_deterministically_and_every_fixture_matches() {
    // FR-019-AC-3: expected_inventory is exactly the five published registries
    // under their five stable prefixes, sorted, and nothing else.
    let mut rebuilt = quire_contract_ir::PUBLIC_CONSTRUCT_TAGS
        .iter()
        .map(|tag| format!("construct:{tag}"))
        .chain(
            DiagnosticCode::ALL
                .iter()
                .map(|code| format!("diagnostic:{}", code.as_str())),
        )
        .chain(
            [
                "option_presence",
                "non_zero_divisor",
                "index_in_bounds",
                "checked_range",
            ]
            .iter()
            .map(|obligation| format!("obligation:{obligation}")),
        )
        .chain(
            quire_contract_ir::CONFORMANCE_BOUNDARIES
                .iter()
                .map(|boundary| format!("boundary:{boundary}")),
        )
        .chain(
            quire_contract_ir::ConformanceOperation::ALL
                .iter()
                .map(|operation| format!("operation:{}", operation.as_str())),
        )
        .collect::<Vec<_>>();
    rebuilt.sort();
    let published = expected_inventory();
    assert_eq!(published, rebuilt);
    assert!(published.windows(2).all(|pair| pair[0] < pair[1]));

    let corpus = corpus();
    let first = run_corpus(&corpus);
    let second = run_corpus(&corpus);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    let rows = rows(&first);
    assert_eq!(rows.len(), input_count(&corpus));
    let mut covered = BTreeSet::new();
    for row in &rows {
        assert_eq!(row["status"], "match");
        assert!(!row["trace_ids"].as_array().unwrap().is_empty());
        covered.extend(
            row["covers"]
                .as_array()
                .unwrap()
                .iter()
                .map(|token| token.as_str().unwrap().to_owned()),
        );
    }
    assert_eq!(covered.into_iter().collect::<Vec<_>>(), published);

    let invalid_utf8 =
        ContractPackage::from_json_bytes(&[0xff], ValidationOptions::strict()).unwrap_err();
    assert_eq!(invalid_utf8[0].code, DiagnosticCode::InvalidWireFormat);
}

/// Tracing: TC-018, FR-018-AC-1, FR-018-AC-2, FR-020-AC-2.
/// StR-003-VC-1.
/// FR-018-AC-1.
/// FR-018-AC-2.
/// FR-020-AC-2.
#[test]
fn tc_018_all_mismatch_kinds_and_exit_classes_are_stable() {
    let scratch = Scratch::corpus("mismatch");

    let package_path = scratch.0.join("expectations/package-constructs.json");
    let mut package = read_json(&package_path);
    package["valid"] = json!(false);
    package["diagnostics"] = json!([{
        "code": "invalid_identifier",
        "severity": "error",
        "path": "mutated"
    }]);
    package["canonical"][0]["bytes_path"] = json!("canonical/package-constructs-1.json");
    package["canonical"][0]["digest"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    package["dependencies"] = json!([{
        "requirement": {"package": "agent-ix/conformance", "requirement": "REQ_alpha", "revision": 1},
        "kind": "input",
        "path": ["mutated"]
    }]);
    write_json(&package_path, &package);

    let migration_path = scratch.0.join("expectations/migration-valid.json");
    let mut migration = read_json(&migration_path);
    migration["migration_receipt"]["target_package_digest"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    write_json(&migration_path, &migration);

    let coverage_path = scratch.0.join("expectations/coverage-shallow.json");
    let mut coverage = read_json(&coverage_path);
    coverage["coverage"] = Value::Null;
    write_json(&coverage_path, &coverage);

    let mismatch = run_corpus(&scratch.0);
    assert_eq!(mismatch.status.code(), Some(1));
    assert!(mismatch.stderr.is_empty());
    let mismatch_rows = rows(&mismatch);
    let package_row = mismatch_rows
        .iter()
        .find(|row| row["fixture_id"] == "package-constructs")
        .unwrap();
    assert_eq!(
        package_row["mismatch_kinds"],
        json!([
            "validity",
            "diagnostics",
            "canonical_bytes",
            "canonical_digest",
            "dependencies"
        ])
    );
    let kinds = mismatch_rows
        .iter()
        .flat_map(|row| {
            row["mismatch_kinds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        kinds,
        [
            "canonical_bytes",
            "canonical_digest",
            "coverage",
            "dependencies",
            "diagnostics",
            "migration_receipt",
            "validity",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );

    assert_eq!(error_code(&runner(&[])), "invalid_invocation");
    assert_eq!(
        error_code(&runner(&["--version", "again"])),
        "invalid_invocation"
    );
    assert_eq!(
        error_code(&runner(&["run", "--manifest", "manifest.json"])),
        "invalid_invocation"
    );

    let malformed_schema = Scratch::corpus("malformed-schema");
    let schema_path = malformed_schema
        .0
        .join("schemas/contract-package-reference-v1.schema.json");
    let mut schema = read_json(&schema_path);
    schema["type"] = json!(17);
    write_json(&schema_path, &schema);
    assert_eq!(
        error_code(&run_corpus(&malformed_schema.0)),
        "invalid_corpus"
    );

    let tightened_schema = Scratch::corpus("tightened-schema");
    let schema_path = tightened_schema
        .0
        .join("schemas/contract-package-reference-v1.schema.json");
    let mut schema = read_json(&schema_path);
    schema["definitions"]["package"]["required"]
        .as_array_mut()
        .unwrap()
        .push(json!("reviewer_probe"));
    write_json(&schema_path, &schema);
    assert_eq!(
        error_code(&run_corpus(&tightened_schema.0)),
        "invalid_corpus",
        "successful semantic packages must be checked against the published schema"
    );

    let missing = Scratch::corpus("missing-expectation");
    fs::remove_file(missing.0.join("expectations/package-reference.json")).unwrap();
    assert_eq!(error_code(&run_corpus(&missing.0)), "fixture_io");

    let unknown_operation = Scratch::corpus("unknown-operation");
    fs::copy(
        unknown_operation.0.join("inputs/package-reference.json"),
        unknown_operation.0.join("inputs/unknown-reference.json"),
    )
    .unwrap();
    assert_eq!(
        error_code(&run_corpus(&unknown_operation.0)),
        "invalid_corpus"
    );

    let uncovered = Scratch::corpus("uncovered");
    for directory in ["inputs", "expectations"] {
        fs::remove_file(uncovered.0.join(directory).join("coverage-digest.json")).unwrap();
    }
    assert_eq!(error_code(&run_corpus(&uncovered.0)), "invalid_corpus");

    let controls = Scratch::corpus("controls");
    let bare = Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
        .current_dir(&controls.0)
        .args(["run", "--corpus", "."])
        .output()
        .unwrap();
    assert!(
        bare.status.success(),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );

    let count = Scratch::corpus("count");
    let prototype = fs::read(count.0.join("inputs/package-reference.json")).unwrap();
    for index in 0..=10_000 {
        fs::write(
            count
                .0
                .join(format!("inputs/package-count-probe-{index}.json")),
            &prototype,
        )
        .unwrap();
    }
    assert_eq!(error_code(&run_corpus(&count.0)), "resource_exhausted");

    let oversized = Scratch::corpus("oversized");
    fs::write(
        oversized.0.join("inputs/package-oversized.json"),
        vec![b' '; 16_777_217],
    )
    .unwrap();
    assert_eq!(error_code(&run_corpus(&oversized.0)), "resource_exhausted");

    let aggregate = Scratch::corpus("aggregate");
    let input = fs::read(
        aggregate
            .0
            .join("inputs/expression-semantic-nodes-over.json"),
    )
    .unwrap();
    let expectation = fs::read(
        aggregate
            .0
            .join("expectations/expression-semantic-nodes-over.json"),
    )
    .unwrap();
    for index in 0..5 {
        let name = format!("expression-aggregate-probe-{index}.json");
        fs::write(aggregate.0.join("inputs").join(&name), &input).unwrap();
        fs::write(aggregate.0.join("expectations").join(&name), &expectation).unwrap();
    }
    assert_eq!(error_code(&run_corpus(&aggregate.0)), "resource_exhausted");

    let mut deeply_nested = vec![b'['; 60_000];
    deeply_nested.push(b'0');
    deeply_nested.extend(std::iter::repeat_n(b']', 60_000));
    let deep_input = Scratch::corpus("deep-input");
    fs::write(
        deep_input.0.join("inputs/package-reference.json"),
        &deeply_nested,
    )
    .unwrap();
    assert_eq!(error_code(&run_corpus(&deep_input.0)), "resource_exhausted");

    let deep_schema = Scratch::corpus("deep-schema");
    fs::write(
        deep_schema
            .0
            .join("schemas/contract-conformance-manifest-v1.schema.json"),
        &deeply_nested,
    )
    .unwrap();
    assert_eq!(
        error_code(&run_corpus(&deep_schema.0)),
        "resource_exhausted"
    );

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        let output = Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
            .arg(OsString::from_vec(vec![0xff]))
            .output()
            .unwrap();
        assert_eq!(error_code(&output), "invalid_invocation");
    }
}

/// Tracing: TC-018, FR-019-AC-2, NFR-003-AC-1.
/// FR-019-AC-2.
#[test]
fn tc_018_semantic_depth_and_collection_edges_preflight_without_panic() {
    let linked_run = catch_unwind(|| quire_contract_ir::run_corpus(&corpus()));
    let linked_results = linked_run
        .expect("the linked runner panicked over the complete corpus")
        .expect("the linked runner rejected the published corpus");
    assert_eq!(linked_results.len(), input_count(&corpus()));
    assert!(linked_results
        .iter()
        .all(|result| result.status() == quire_contract_ir::FixtureStatus::Match));

    let mut deeply_nested_package = String::from(
        r#"{"id":"agent-ix/depth","schema_version":{"major":1,"minor":1},"source":{"document":"depth","revision":1},"requirements":[],"ignored":"#,
    );
    deeply_nested_package.extend(std::iter::repeat_n('[', 2_048));
    deeply_nested_package.push('0');
    deeply_nested_package.extend(std::iter::repeat_n(']', 2_048));
    deeply_nested_package.push('}');
    let deep_decode = catch_unwind(|| {
        ContractPackage::from_json_str(&deeply_nested_package, ValidationOptions::strict())
    })
    .expect("deep package decoding panicked")
    .unwrap_err();
    assert_eq!(deep_decode[0].code, DiagnosticCode::InvalidWireFormat);

    let at_wire_depth = format!(
        "{}0{}",
        "[".repeat(MAX_WIRE_JSON_DEPTH as usize),
        "]".repeat(MAX_WIRE_JSON_DEPTH as usize)
    );
    let over_wire_depth = format!(
        "{}0{}",
        "[".repeat(MAX_WIRE_JSON_DEPTH as usize + 1),
        "]".repeat(MAX_WIRE_JSON_DEPTH as usize + 1)
    );
    let at_limit = catch_unwind(|| {
        ContractPackage::from_json_str(&at_wire_depth, ValidationOptions::strict())
    })
    .expect("at-limit wire package decoding panicked")
    .unwrap_err();
    assert_eq!(at_limit[0].code, DiagnosticCode::InvalidWireFormat);
    assert_eq!(at_limit[0].path, "document");

    let over_limit = catch_unwind(|| {
        ContractPackage::from_json_str(&over_wire_depth, ValidationOptions::strict())
    })
    .expect("over-limit wire package decoding panicked")
    .unwrap_err();
    assert_eq!(over_limit[0].code, DiagnosticCode::InvalidWireFormat);
    assert_eq!(over_limit[0].path, "document.nesting");

    let owner = RequirementRef::new(
        PackageId::new("agent-ix/conformance").unwrap(),
        RequirementId::new("REQ_limits").unwrap(),
        RequirementRevision::new(1).unwrap(),
    );
    let source = SourceIdentity::new(
        SourceDocumentId::new("limits").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let span = SourceSpan::new(
        SourceLocation::new(source.clone(), 1, 1, 0).unwrap(),
        SourceLocation::new(source, 1, 2, 1).unwrap(),
    )
    .unwrap();

    let nested = |depth: u32| {
        let mut value = ValueType::Boolean;
        for _ in 1..depth {
            value = ValueType::option(value);
        }
        value
    };
    let at_depth = DeclarationEnvironment::new(
        owner.clone(),
        vec![],
        vec![ValueDeclaration::new(
            SymbolName::new("depth_ok").unwrap(),
            ValueDeclarationKind::Input,
            nested(MAX_SEMANTIC_DEPTH),
            span.clone(),
        )],
        vec![],
    );
    assert!(at_depth.is_ok());
    let over_depth = DeclarationEnvironment::new(
        owner.clone(),
        vec![],
        vec![ValueDeclaration::new(
            SymbolName::new("depth_bad").unwrap(),
            ValueDeclarationKind::Input,
            nested(MAX_SEMANTIC_DEPTH + 1),
            span.clone(),
        )],
        vec![],
    )
    .unwrap_err();
    assert_eq!(over_depth[0].code, DiagnosticCode::SemanticInputTooLarge);

    let declarations = |count: u32| {
        (0..count)
            .map(|index| {
                ValueDeclaration::new(
                    SymbolName::new(format!("value_{index}")).unwrap(),
                    ValueDeclarationKind::Input,
                    ValueType::Boolean,
                    span.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert!(DeclarationEnvironment::new(
        owner.clone(),
        vec![],
        declarations(MAX_SEMANTIC_COLLECTION_ITEMS),
        vec![],
    )
    .is_ok());
    let over_collection = DeclarationEnvironment::new(
        owner,
        vec![],
        declarations(MAX_SEMANTIC_COLLECTION_ITEMS + 1),
        vec![],
    )
    .unwrap_err();
    assert_eq!(
        over_collection[0].code,
        DiagnosticCode::SemanticInputTooLarge
    );
}

/// TC-018. FR-018-AC-3.
#[test]
fn tc_018_wire_depth_controls_ignore_quoted_delimiters_and_pin_literal_cliff() {
    // Independent authored counts, not the producer's scanner or depth constant.
    // The object contributes one level; its escaped string contributes none.
    let quoted =
        serde_json::to_string(&format!("\\\"{}{}", "[".repeat(600), "}".repeat(600))).unwrap();
    for (arrays, expected_path) in [(575, "document"), (576, "document.nesting")] {
        let document = format!(
            "{}{{\"text\":{quoted}}}{}",
            "[".repeat(arrays),
            "]".repeat(arrays)
        );
        let diagnostics =
            ContractPackage::from_json_str(&document, ValidationOptions::strict()).unwrap_err();
        assert_eq!(diagnostics[0].code, DiagnosticCode::InvalidWireFormat);
        assert_eq!(diagnostics[0].path, expected_path);
    }
}
