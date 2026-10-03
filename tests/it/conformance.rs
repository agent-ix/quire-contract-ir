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

fn schemas() -> PathBuf {
    repository().join("schemas")
}

fn run_with(corpus: &Path, schemas: &Path) -> Output {
    runner(&[
        "run",
        "--corpus",
        corpus.to_str().unwrap(),
        "--schemas",
        schemas.to_str().unwrap(),
    ])
}

fn run_corpus(path: &Path) -> Output {
    run_with(path, &schemas())
}

/// A scratch copy of the corpus at `<root>/contract-v0.1` with the schemas it
/// runs against at `<root>/schemas`, so a test can change either.
struct Scratch(PathBuf);

impl Scratch {
    fn corpus(label: &str) -> Self {
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

    fn root(&self) -> &Path {
        self.0.parent().unwrap()
    }

    fn schemas(&self) -> PathBuf {
        self.root().join("schemas")
    }

    fn run(&self) -> Output {
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

/// Tracing: TC-018, FR-018-AC-1, FR-019-AC-1, FR-019-AC-3, FR-020-AC-1,
/// NFR-001-AC-1.
/// NFR-001-AC-1.
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

    let coverage_path = scratch.0.join("expectations/coverage-shallow.json");
    let mut coverage = read_json(&coverage_path);
    coverage["coverage"] = Value::Null;
    write_json(&coverage_path, &coverage);

    let mismatch = scratch.run();
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
        .schemas()
        .join("contract-package-reference-v1.schema.json");
    let mut schema = read_json(&schema_path);
    schema["type"] = json!(17);
    write_json(&schema_path, &schema);
    assert_eq!(error_code(&malformed_schema.run()), "invalid_corpus");

    let tightened_schema = Scratch::corpus("tightened-schema");
    let schema_path = tightened_schema
        .schemas()
        .join("contract-package-reference-v1.schema.json");
    let mut schema = read_json(&schema_path);
    schema["definitions"]["package"]["required"]
        .as_array_mut()
        .unwrap()
        .push(json!("reviewer_probe"));
    write_json(&schema_path, &schema);
    assert_eq!(
        error_code(&tightened_schema.run()),
        "invalid_corpus",
        "successful semantic packages must be checked against the published schema"
    );

    let missing = Scratch::corpus("missing-expectation");
    fs::remove_file(missing.0.join("expectations/package-reference.json")).unwrap();
    assert_eq!(error_code(&missing.run()), "fixture_io");

    let unknown_operation = Scratch::corpus("unknown-operation");
    fs::copy(
        unknown_operation.0.join("inputs/package-reference.json"),
        unknown_operation.0.join("inputs/unknown-reference.json"),
    )
    .unwrap();
    assert_eq!(error_code(&unknown_operation.run()), "invalid_corpus");

    let uncovered = Scratch::corpus("uncovered");
    for directory in ["inputs", "expectations"] {
        fs::remove_file(uncovered.0.join(directory).join("coverage-digest.json")).unwrap();
    }
    assert_eq!(error_code(&uncovered.run()), "invalid_corpus");

    let controls = Scratch::corpus("controls");
    let bare = Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
        .current_dir(&controls.0)
        .args(["run", "--corpus", ".", "--schemas"])
        .arg(controls.schemas())
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
    assert_eq!(error_code(&count.run()), "resource_exhausted");

    let oversized = Scratch::corpus("oversized");
    fs::write(
        oversized.0.join("inputs/package-oversized.json"),
        vec![b' '; 16_777_217],
    )
    .unwrap();
    assert_eq!(error_code(&oversized.run()), "resource_exhausted");

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
    assert_eq!(error_code(&aggregate.run()), "resource_exhausted");

    let mut deeply_nested = vec![b'['; 60_000];
    deeply_nested.push(b'0');
    deeply_nested.extend(std::iter::repeat_n(b']', 60_000));
    let deep_input = Scratch::corpus("deep-input");
    fs::write(
        deep_input.0.join("inputs/package-reference.json"),
        &deeply_nested,
    )
    .unwrap();
    assert_eq!(error_code(&deep_input.run()), "resource_exhausted");

    let deep_schema = Scratch::corpus("deep-schema");
    fs::write(
        deep_schema
            .schemas()
            .join("contract-conformance-fixture-v1.schema.json"),
        &deeply_nested,
    )
    .unwrap();
    assert_eq!(error_code(&deep_schema.run()), "resource_exhausted");

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

/// Tracing: TC-018, FR-018-AC-1, FR-020-AC-2.
/// FR-018-AC-1.
/// FR-020-AC-2.
#[test]
fn tc_018_runner_refuses_unsafe_paths_foreign_schemas_and_stray_entries() {
    let version = runner(&["--version"]);
    assert_eq!(version.status.code(), Some(0));
    assert!(version.stderr.is_empty());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        format!(
            "quire-contract-ir {} quire.contract.conformance-jsonl/v1\n",
            env!("CARGO_PKG_VERSION")
        )
    );

    let traversal = Scratch::corpus("traversal");
    let path = traversal.0.join("expectations/package-constructs.json");
    let mut expectation = read_json(&path);
    expectation["canonical"][0]["bytes_path"] = json!("../escape.json");
    write_json(&path, &expectation);
    let output = traversal.run();
    assert_eq!(error_code(&output), "unsafe_path");
    let root = traversal.root().to_str().unwrap().to_owned();
    assert!(!String::from_utf8_lossy(&output.stderr).contains(&root));

    #[cfg(unix)]
    {
        let symlink = Scratch::corpus("symlink");
        let outside = symlink.root().join("outside.json");
        fs::copy(symlink.0.join("inputs/package-reference.json"), &outside).unwrap();
        std::os::unix::fs::symlink(&outside, symlink.0.join("inputs/package-escape.json")).unwrap();
        let output = symlink.run();
        assert_eq!(error_code(&output), "unsafe_path");
        let root = symlink.root().to_str().unwrap().to_owned();
        assert!(!String::from_utf8_lossy(&output.stderr).contains(&root));
    }

    let foreign = Scratch::corpus("foreign-schema");
    let path = foreign
        .schemas()
        .join("contract-package-reference-v1.schema.json");
    let mut schema = read_json(&path);
    schema["$id"] = json!("https://example.invalid/other-package.schema.json");
    write_json(&path, &schema);
    assert_eq!(error_code(&foreign.run()), "unsupported_profile");

    let stray = Scratch::corpus("stray-input");
    fs::write(stray.0.join("inputs/notes.txt"), b"not a fixture").unwrap();
    assert_eq!(error_code(&stray.run()), "invalid_corpus");

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        let non_utf8 = Scratch::corpus("non-utf8-input");
        fs::write(
            non_utf8
                .0
                .join("inputs")
                .join(OsString::from_vec(b"package-\xff.json".to_vec())),
            b"{}",
        )
        .unwrap();
        assert_eq!(error_code(&non_utf8.run()), "invalid_corpus");
    }

    let orphan = Scratch::corpus("orphan-expectation");
    fs::copy(
        orphan.0.join("expectations/package-reference.json"),
        orphan.0.join("expectations/package-orphan.json"),
    )
    .unwrap();
    assert_eq!(error_code(&orphan.run()), "invalid_corpus");
}

/// Tracing: TC-018, FR-019-AC-2, NFR-003-AC-1.
/// FR-019-AC-2.
#[test]
fn tc_018_semantic_depth_and_collection_edges_preflight_without_panic() {
    let linked_run = catch_unwind(|| quire_contract_ir::run_corpus(&corpus(), &schemas()));
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

/// Tracing: TC-018, FR-018-AC-3.
/// FR-018-AC-3.
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

/// `definition` of the schema file `file` as a validator of its own.
fn definition_validator(file: &str, definition: &str) -> jsonschema::JSONSchema {
    let mut schema = read_json(&schemas().join(file));
    let definitions = schema["definitions"].take();
    let wrapper = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "definitions": definitions,
        "$ref": format!("#/definitions/{definition}"),
    });
    jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .compile(&wrapper)
        .unwrap_or_else(|error| panic!("{file}#{definition} does not compile: {error}"))
}

fn schema_span() -> Value {
    let source = json!({"document": "doc", "revision": 1});
    json!({
        "start": {"source": source, "line": 1, "column": 1, "byte_offset": 0},
        "end": {"source": source, "line": 1, "column": 2, "byte_offset": 1},
    })
}

/// Builds the schema instance holding one spelling of a member, and says
/// whether the instance is a `valueType` (true) or an `expression` (false).
type Instance<'a> = Box<dyn Fn(&Value) -> (bool, Value) + 'a>;

/// The accepted and refused spellings of each of the eight integer members
/// (FR-013-AC-5, FR-020-AC-3).
fn integer_string_spellings() -> (Vec<Value>, Vec<Value>) {
    let accepted = ["-9223372036854775808", "0", "9223372036854775807"]
        .map(Value::from)
        .to_vec();
    let mut refused = vec![
        json!(0),
        json!(1),
        json!(1.0),
        json!(9_223_372_036_854_775_807_i64),
    ];
    refused.extend(["+1", "01", "-0", "", "1.0", "1e3", " 1"].map(Value::from));
    (accepted, refused)
}

/// Tracing: TC-018, FR-013-AC-5, FR-020-AC-3.
/// FR-013-AC-5.
/// FR-020-AC-3.
#[ix_trace_rs::trace("TC-018", "FR-013-AC-5", "FR-020-AC-3")]
#[test]
fn tc_018_the_fixture_schema_takes_the_eight_members_as_integer_strings() {
    let (accepted, refused) = integer_string_spellings();
    let fixture = "contract-conformance-fixture-v1.schema.json";
    let value_type = definition_validator(fixture, "valueType");
    let expression = definition_validator(fixture, "expression");
    let integer_type = |member: &str, spelling: &Value| {
        let mut value = json!({"kind": "integer", "domain": "signed", "minimum": "0",
                               "maximum": "0", "overflow": "reject"});
        value[member] = spelling.clone();
        value
    };
    let rational_type = |member: &str, spelling: &Value| {
        let mut value = json!({"kind": "rational", "numerator_minimum": "0",
                               "numerator_maximum": "0", "maximum_denominator": "1"});
        value[member] = spelling.clone();
        value
    };
    let integer_literal = |spelling: &Value| {
        json!({"node": "integer_literal", "value": spelling, "source": schema_span(),
               "value_type": integer_type("minimum", &json!("0"))})
    };
    let rational_literal = |member: &str, spelling: &Value| {
        let mut value = json!({"node": "rational_literal", "numerator": "1", "denominator": "1",
                               "source": schema_span(),
                               "value_type": rational_type("maximum_denominator", &json!("1"))});
        value[member] = spelling.clone();
        value
    };
    let cases: Vec<(&str, Instance<'_>)> = vec![
        (
            "IntegerType.minimum",
            Box::new(move |s| (true, integer_type("minimum", s))),
        ),
        (
            "IntegerType.maximum",
            Box::new(move |s| (true, integer_type("maximum", s))),
        ),
        (
            "RationalType.numerator_minimum",
            Box::new(move |s| (true, rational_type("numerator_minimum", s))),
        ),
        (
            "RationalType.numerator_maximum",
            Box::new(move |s| (true, rational_type("numerator_maximum", s))),
        ),
        (
            "RationalType.maximum_denominator",
            Box::new(move |s| (true, rational_type("maximum_denominator", s))),
        ),
        (
            "IntegerLiteral.value",
            Box::new(move |s| (false, integer_literal(s))),
        ),
        (
            "RationalLiteral.numerator",
            Box::new(move |s| (false, rational_literal("numerator", s))),
        ),
        (
            "RationalLiteral.denominator",
            Box::new(move |s| (false, rational_literal("denominator", s))),
        ),
    ];
    for (name, build) in &cases {
        for spelling in accepted.iter().chain(&refused) {
            let (is_type, instance) = build(spelling);
            let validator = if is_type { &value_type } else { &expression };
            assert_eq!(
                validator.is_valid(&instance),
                accepted.contains(spelling),
                "{name} = {spelling}"
            );
        }
    }
}

/// Tracing: TC-018, FR-020-AC-3.
/// FR-020-AC-3.
#[ix_trace_rs::trace("TC-018", "FR-020-AC-3")]
#[test]
fn tc_018_the_package_schemas_bound_a_revision_and_a_byte_offset_at_two_to_the_53() {
    const BOUND: u64 = 9_007_199_254_740_992;
    let package = read_json(&corpus().join("inputs/package-reference.json"));
    for file in [
        "contract-package-reference-v1.schema.json",
        "contract-conformance-fixture-v1.schema.json",
    ] {
        let validator = definition_validator(file, "package");
        assert!(
            validator.is_valid(&package),
            "{file}: the fixture package is valid"
        );
        for pointer in [
            "/source/revision",
            "/requirements/0/revision",
            "/requirements/0/source/start/byte_offset",
            "/requirements/0/source/end/byte_offset",
            "/requirements/0/clauses/0/source/end/byte_offset",
        ] {
            for (spelling, valid) in [(BOUND, true), (BOUND + 1, false)] {
                let mut candidate = package.clone();
                *candidate.pointer_mut(pointer).unwrap() = json!(spelling);
                assert_eq!(
                    validator.is_valid(&candidate),
                    valid,
                    "{file}: {pointer} = {spelling}"
                );
            }
        }
    }
}

/// Tracing: TC-018, FR-020-AC-3.
/// FR-020-AC-3.
#[ix_trace_rs::trace("TC-018", "FR-020-AC-3")]
#[test]
fn tc_018_a_decoder_fixture_reaches_the_decoder_and_a_schema_input_does_not() {
    let corpus = corpus();
    let output = run_corpus(&corpus);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = rows(&output);
    for (id, code) in [
        (
            "expression-invalid-wire-number-bound",
            "invalid_wire_format",
        ),
        (
            "expression-invalid-wire-number-literal",
            "invalid_wire_format",
        ),
        (
            "expression-invalid-wire-string-grammar",
            "invalid_wire_format",
        ),
        ("expression-invalid-numeric-range", "invalid_numeric_bounds"),
        (
            "package-invalid-requirement-revision-over",
            "invalid_requirement_revision",
        ),
        (
            "package-invalid-source-revision-over",
            "invalid_source_revision",
        ),
        ("package-invalid-span-offset-over", "invalid_source_span"),
    ] {
        let expectation = read_json(&corpus.join(format!("expectations/{id}.json")));
        assert_eq!(
            expectation["diagnostics"][0]["code"], code,
            "{id}: expectation"
        );
        let row = rows
            .iter()
            .find(|row| row["fixture_id"] == id)
            .unwrap_or_else(|| panic!("{id} is in the corpus"));
        assert_eq!(row["status"], "match", "{id}");
        assert_eq!(
            row["actual"]["diagnostics"][0]["code"], code,
            "{id}: actual"
        );
        if id.starts_with("expression-invalid-wire-") {
            // The input is the text the decoder reads, with the number in it.
            let input = read_json(&corpus.join(format!("inputs/{id}.json")));
            assert!(input["document_json"].is_string(), "{id}");
        }
    }

    // The same number in an ordinary operation input is refused by the schema
    // before any decoder runs: the corpus is invalid.
    let scratch = Scratch::corpus("number-member");
    let (_, refused) = integer_string_spellings();
    for (fixture, pointer) in [
        ("expression-integer-literal", "/expected_type/minimum"),
        ("expression-integer-literal", "/expected_type/maximum"),
        ("expression-integer-literal", "/expression/value"),
        (
            "expression-rational-literal",
            "/expected_type/numerator_minimum",
        ),
        (
            "expression-rational-literal",
            "/expected_type/numerator_maximum",
        ),
        (
            "expression-rational-literal",
            "/expected_type/maximum_denominator",
        ),
        ("expression-rational-literal", "/expression/numerator"),
        ("expression-rational-literal", "/expression/denominator"),
    ] {
        let path = scratch.0.join(format!("inputs/{fixture}.json"));
        let original = read_json(&path);
        for spelling in [&refused[0], &refused[2], &refused[4]] {
            let mut changed = original.clone();
            *changed
                .pointer_mut(pointer)
                .expect("the member is in the fixture") = spelling.clone();
            write_json(&path, &changed);
            assert_eq!(
                error_code(&scratch.run()),
                "invalid_corpus",
                "{fixture} {pointer} = {spelling}"
            );
        }
        write_json(&path, &original);
    }
    assert!(scratch.run().status.success());
}

/// Every canonical file of the published corpus whose object holds one of the
/// eight members equals the bytes written out here, or holds them only as
/// strings (FR-020-AC-3, FR-016-AC-5).
///
/// Tracing: TC-018, FR-020-AC-3.
/// FR-020-AC-3.
#[ix_trace_rs::trace("TC-018", "FR-020-AC-3", "FR-016-AC-5")]
#[test]
fn tc_018_the_recorded_canonical_files_spell_the_eight_members_as_strings() {
    const INTEGER: &str =
        "{\"domain\":\"signed\",\"maximum\":\"10\",\"minimum\":\"-10\",\"overflow\":\"reject\"}";
    let canonical = corpus().join("canonical");
    let read = |name: &str| fs::read_to_string(canonical.join(name)).unwrap();
    assert_eq!(
        read("expression-integer-literal-1.json"),
        format!(
            "{{\"kind\":\"expression\",\"profile\":\"quire.contract.canonical-json/v1\",\
             \"value\":{{\"result_type\":{{\"kind\":\"integer\",\"value\":{INTEGER}}},\
             \"tree\":{{\"kind\":{{\"node\":\"integer_literal\",\"value\":\"1\",\
             \"value_type\":{INTEGER}}}}}}}}}"
        )
    );
    const RATIONAL: &str = concat!(
        "{\"maximum_denominator\":\"10\",\"numerator_maximum\":\"10\",",
        "\"numerator_minimum\":\"-10\"}"
    );
    assert_eq!(
        read("expression-rational-literal-1.json"),
        format!(
            "{{\"kind\":\"expression\",\"profile\":\"quire.contract.canonical-json/v1\",\
             \"value\":{{\"result_type\":{{\"kind\":\"rational\",\"value\":{RATIONAL}}},\
             \"tree\":{{\"kind\":{{\"denominator\":\"1\",\"node\":\"rational_literal\",\
             \"numerator\":\"2\",\"value_type\":{RATIONAL}}}}}}}}}"
        )
    );
    assert_eq!(
        read("expression-collection-literal-1.json"),
        format!(
            "{{\"kind\":\"expression\",\"profile\":\"quire.contract.canonical-json/v1\",\
             \"value\":{{\"result_type\":{{\"kind\":\"collection\",\"value\":\
             {{\"element\":{{\"kind\":\"integer\",\"value\":{INTEGER}}},\"maximum_items\":4}}}},\
             \"tree\":{{\"kind\":{{\"items\":[{{\"kind\":{{\"node\":\"integer_literal\",\
             \"value\":\"1\",\"value_type\":{INTEGER}}}}}],\"node\":\"collection_literal\",\
             \"value_type\":{{\"element\":{{\"kind\":\"integer\",\"value\":{INTEGER}}},\
             \"maximum_items\":4}}}}}}}}}}"
        )
    );
    // The widest members: an integer type of the full `i64` range and a
    // rational type whose denominator bound is `i64::MAX`.
    let edges = read("expression-numeric-edges-0.json");
    for member in [
        "\"maximum\":\"9223372036854775807\",\"minimum\":\"-9223372036854775808\"",
        "\"maximum_denominator\":\"9223372036854775807\",\"numerator_maximum\":\"1\",\
         \"numerator_minimum\":\"-1\"",
        "\"maximum_items\":4294967295",
    ] {
        assert!(edges.contains(member), "numeric edges hold {member}");
    }

    let names = [
        "minimum",
        "maximum",
        "numerator_minimum",
        "numerator_maximum",
        "maximum_denominator",
        "value",
        "numerator",
        "denominator",
    ];
    let mut holding = 0;
    for entry in fs::read_dir(&canonical).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        let mut holds = false;
        for name in names {
            let needle = format!("\"{name}\":");
            for (at, _) in text.match_indices(&needle) {
                let next = text[at + needle.len()..].chars().next().unwrap();
                assert!(
                    !(next.is_ascii_digit() || next == '-'),
                    "{}: `{needle}` is followed by `{next}`",
                    path.display()
                );
                holds |= next == '"' && name != "value";
            }
        }
        holding += usize::from(holds);
    }
    assert_eq!(
        holding, 47,
        "47 canonical files hold one of the eight members"
    );
}

/// A successful expression input that arrives as `document_json` is checked
/// against the fixture schema like a plain one is on load, as a successful
/// package document is: the decoder ignores an unknown member of an
/// expression node, the schema does not.
///
/// Tracing: TC-018, FR-020-AC-3.
/// FR-020-AC-3.
#[ix_trace_rs::trace("TC-018", "FR-020-AC-3")]
#[test]
fn tc_018_a_successful_expression_document_is_checked_against_the_schema() {
    let scratch = Scratch::corpus("expression-document");
    let request = read_json(&scratch.0.join("inputs/expression-boolean-literal.json"));
    let expectation = read_json(
        &scratch
            .0
            .join("expectations/expression-boolean-literal.json"),
    );
    for id in ["control", "extra"] {
        write_json(
            &scratch
                .0
                .join(format!("expectations/expression-document-{id}.json")),
            &expectation,
        );
    }
    write_json(
        &scratch.0.join("inputs/expression-document-control.json"),
        &json!({"document_json": request.to_string()}),
    );
    // The inputs directory holds the control only, so its run reaches the
    // fixtures; the extra expectation is an orphan until its input is written.
    fs::remove_file(
        scratch
            .0
            .join("expectations/expression-document-extra.json"),
    )
    .unwrap();
    let output = scratch.run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let control = rows(&output)
        .into_iter()
        .find(|row| row["fixture_id"] == "expression-document-control")
        .expect("the control fixture ran");
    assert_eq!(control["status"], "match");
    assert_eq!(control["actual"]["valid"], true);

    let mut extra = request;
    extra["expression"]["extra"] = json!(1);
    write_json(
        &scratch.0.join("inputs/expression-document-extra.json"),
        &json!({"document_json": extra.to_string()}),
    );
    write_json(
        &scratch
            .0
            .join("expectations/expression-document-extra.json"),
        &expectation,
    );
    assert_eq!(error_code(&scratch.run()), "invalid_corpus");
}
