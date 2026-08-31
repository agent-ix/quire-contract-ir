use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    panic::catch_unwind,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use jsonschema::{Draft, JSONSchema};
use quire_contract_ir::{
    expected_inventory, CanonicalProfile, ContractPackage, DeclarationEnvironment, DiagnosticCode,
    PackageId, RequirementId, RequirementRef, RequirementRevision, SourceDocumentId,
    SourceIdentity, SourceLocation, SourceRevision, SourceSpan, SymbolName, ValidationOptions,
    ValueDeclaration, ValueDeclarationKind, ValueType, CONFORMANCE_BOUNDARIES,
    MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES, PUBLIC_CONSTRUCT_TAGS,
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

fn run_manifest(path: &Path) -> Output {
    runner(&["run", "--manifest", path.to_str().unwrap()])
}

struct Scratch(PathBuf);

impl Scratch {
    fn corpus() -> Self {
        let path = std::env::temp_dir().join(format!(
            "quire-contract-ir-tc018-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        copy_tree(&corpus(), &path);
        Self(path)
    }

    fn manifest(&self) -> PathBuf {
        self.0.join("manifest.json")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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

fn error_code(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["protocol"], "quire.contract.conformance-jsonl/v1");
    error["code"].as_str().unwrap().to_owned()
}

/// Tracing: TC-018, FR-018-AC-1, FR-019-AC-1, FR-020-AC-1.
/// TC-019.
/// FR-018-AC-1.
/// FR-019-AC-1.
/// FR-020-AC-1.
#[test]
fn tc_018_published_schema_inventory_sidecars_and_runner_are_exact() {
    let root = repository();
    let corpus = corpus();
    assert_eq!(
        fs::read(root.join("schemas/contract-package-reference-v1.schema.json")).unwrap(),
        fs::read(corpus.join("schemas/contract-package-reference-v1.schema.json")).unwrap()
    );
    assert_eq!(
        fs::read(root.join("schemas/contract-conformance-manifest-v1.schema.json")).unwrap(),
        fs::read(corpus.join("schemas/contract-conformance-manifest-v1.schema.json")).unwrap()
    );

    for schema in [
        root.join("schemas/contract-package-reference-v1.schema.json"),
        root.join("schemas/contract-conformance-manifest-v1.schema.json"),
    ] {
        let value = read_json(&schema);
        JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&value)
            .unwrap();
    }

    let inventory: Vec<String> =
        serde_json::from_slice(&fs::read(corpus.join("inventory.json")).unwrap()).unwrap();
    assert_eq!(inventory, expected_inventory());
    assert!(PUBLIC_CONSTRUCT_TAGS
        .windows(2)
        .all(|pair| pair[0] < pair[1]));
    assert!(CONFORMANCE_BOUNDARIES
        .windows(2)
        .all(|pair| pair[0] < pair[1]));
    assert_eq!(MAX_SEMANTIC_NODES, 25_000);
    assert_eq!(MAX_SEMANTIC_DEPTH, 256);
    assert_eq!(MAX_SEMANTIC_COLLECTION_ITEMS, 10_000);
    assert_eq!(
        CanonicalProfile::V1.as_str(),
        "quire.contract.canonical-json/v1"
    );
    assert!(ValidationOptions::strict().is_strict());
    assert_eq!(
        DiagnosticCode::ALL.last(),
        Some(&DiagnosticCode::SemanticInputTooLarge)
    );

    for entry in walk_files(&corpus) {
        if entry.extension().and_then(|value| value.to_str()) == Some("sha256")
            || entry.file_name().and_then(|value| value.to_str()) == Some("README.md")
        {
            continue;
        }
        let sidecar = PathBuf::from(format!("{}.sha256", entry.display()));
        assert!(sidecar.is_file(), "missing sidecar for {}", entry.display());
        verify_sidecar(&entry, &sidecar);
    }
    for name in [
        "contract-package-reference-v1.schema.json",
        "contract-conformance-manifest-v1.schema.json",
    ] {
        let schema = root.join("schemas").join(name);
        verify_sidecar(
            &schema,
            &PathBuf::from(format!("{}.sha256", schema.display())),
        );
    }
    for entry in walk_files(&corpus.join("canonical")) {
        if entry.extension().and_then(|value| value.to_str()) == Some("json") {
            assert_ne!(fs::read(entry).unwrap().last(), Some(&b'\n'));
        }
    }

    let manifest = corpus.join("manifest.json");
    let first = run_manifest(&manifest);
    let second = run_manifest(&manifest);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    let rows = first
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| serde_json::from_slice::<Value>(row).unwrap())
        .collect::<Vec<_>>();
    let fixture_count = read_json(&manifest)["fixtures"].as_array().unwrap().len();
    assert_eq!(rows.len(), fixture_count);
    assert!(rows.iter().all(|row| row["status"] == "match"));

    let package_schema_value =
        read_json(&root.join("schemas/contract-package-reference-v1.schema.json"));
    let package_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&package_schema_value)
        .unwrap();
    let manifest_value = read_json(&manifest);
    let mut schema_negative = BTreeSet::new();
    for fixture in manifest_value["fixtures"].as_array().unwrap() {
        let operation = fixture["operation"].as_str().unwrap();
        if operation == "expression" {
            continue;
        }
        let input = read_json(&corpus.join(fixture["input"].as_str().unwrap()));
        let package = if operation == "package" {
            input.get("package").unwrap_or(&input)
        } else {
            &input["package"]
        };
        let schema_valid = package_schema.is_valid(package);
        let expectation = read_json(&corpus.join(fixture["expectation"].as_str().unwrap()));
        let semantic_success = expectation["valid"].as_bool() == Some(true)
            || expectation
                .get("coverage")
                .is_some_and(|coverage| !coverage.is_null());
        if semantic_success {
            assert!(
                schema_valid,
                "successful fixture {} diverges from the package schema",
                fixture["id"]
            );
        }
        if !schema_valid {
            schema_negative.insert(fixture["id"].as_str().unwrap().to_owned());
        }
    }
    assert_eq!(
        schema_negative,
        [
            "migration-unregistered",
            "migration-unsupported",
            "package-invalid-identifier",
            "package-invalid-namespace",
            "package-invalid-requirement-revision",
            "package-invalid-schema",
            "package-invalid-source-revision",
            "package-malformed-reference",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );

    let semantic_max = read_json(&corpus.join("inputs/expression-semantic-nodes-maximum.json"));
    let semantic_over = read_json(&corpus.join("inputs/expression-semantic-nodes-over.json"));
    let mut one_past = semantic_max;
    one_past["values"][0]["value_type"] = json!({"kind": "option", "value": {"kind": "boolean"}});
    assert_eq!(
        semantic_over, one_past,
        "semantic-node over fixture must add exactly one nested type node"
    );

    let version = runner(&["--version"]);
    assert!(version.status.success());
    assert!(version.stderr.is_empty());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "quire-contract-ir 0.1.0 quire.contract.conformance-jsonl/v1\n"
    );

    let invalid_utf8 =
        ContractPackage::from_json_bytes(&[0xff], ValidationOptions::strict()).unwrap_err();
    assert_eq!(invalid_utf8[0].code, DiagnosticCode::InvalidWireFormat);
}

/// Tracing: TC-018, FR-018-AC-2, FR-020-AC-2.
/// FR-018-AC-2.
/// FR-020-AC-2.
#[test]
fn tc_018_all_mismatch_kinds_and_exit_classes_are_stable() {
    let scratch = Scratch::corpus();

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

    let coverage_path = scratch.0.join("expectations/coverage-complete.json");
    let mut coverage = read_json(&coverage_path);
    coverage["coverage"] = Value::Null;
    write_json(&coverage_path, &coverage);

    let mismatch = run_manifest(&scratch.manifest());
    assert_eq!(mismatch.status.code(), Some(1));
    assert!(mismatch.stderr.is_empty());
    let mismatch_rows = mismatch
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| serde_json::from_slice::<Value>(row).unwrap())
        .collect::<Vec<_>>();
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

    let invalid = Scratch::corpus();
    let mut manifest = read_json(&invalid.manifest());
    manifest["package_schema"]["sha256"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    write_json(&invalid.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&invalid.manifest())),
        "invalid_manifest"
    );

    let malformed_schema = Scratch::corpus();
    let schema_path = malformed_schema
        .0
        .join("schemas/contract-package-reference-v1.schema.json");
    let mut schema = read_json(&schema_path);
    schema["type"] = json!(17);
    write_json(&schema_path, &schema);
    let mut manifest = read_json(&malformed_schema.manifest());
    manifest["package_schema"]["sha256"] = json!(quire_contract_ir::hex_digest(
        &fs::read(&schema_path).unwrap()
    ));
    write_json(&malformed_schema.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&malformed_schema.manifest())),
        "invalid_manifest"
    );

    let false_coverage = Scratch::corpus();
    let mut manifest = read_json(&false_coverage.manifest());
    let covers = manifest["fixtures"][0]["covers"].as_array_mut().unwrap();
    covers.push(json!("diagnostic:arity_mismatch"));
    covers.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    write_json(&false_coverage.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&false_coverage.manifest())),
        "invalid_manifest"
    );

    let unsupported = Scratch::corpus();
    let mut manifest = read_json(&unsupported.manifest());
    manifest["canonical_profile"] = json!("unknown/profile");
    write_json(&unsupported.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&unsupported.manifest())),
        "unsupported_profile"
    );

    let unsafe_path = Scratch::corpus();
    let mut manifest = read_json(&unsafe_path.manifest());
    manifest["package_schema"]["path"] = json!("../escape.json");
    write_json(&unsafe_path.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&unsafe_path.manifest())),
        "unsafe_path"
    );

    let missing = Scratch::corpus();
    let mut manifest = read_json(&missing.manifest());
    manifest["fixtures"][0]["input"] = json!("inputs/missing.json");
    write_json(&missing.manifest(), &manifest);
    assert_eq!(error_code(&run_manifest(&missing.manifest())), "fixture_io");

    let exhausted = Scratch::corpus();
    fs::write(exhausted.manifest(), vec![b' '; 16_777_217]).unwrap();
    assert_eq!(
        error_code(&run_manifest(&exhausted.manifest())),
        "resource_exhausted"
    );

    let controls = Scratch::corpus();
    let baseline = read_json(&controls.manifest());

    let mut manifest = baseline.clone();
    manifest["fixtures"][1]["id"] = manifest["fixtures"][0]["id"].clone();
    write_json(&controls.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&controls.manifest())),
        "invalid_manifest"
    );

    let mut manifest = baseline.clone();
    let covers = manifest["fixtures"][0]["covers"].as_array_mut().unwrap();
    covers.push(json!("boundary:not-registered"));
    covers.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    write_json(&controls.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&controls.manifest())),
        "invalid_manifest"
    );

    let mut manifest = baseline.clone();
    let covers = manifest["fixtures"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find_map(|fixture| {
            let covers = fixture["covers"].as_array_mut().unwrap();
            (covers.len() > 1).then_some(covers)
        })
        .unwrap();
    covers.swap(0, 1);
    write_json(&controls.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&controls.manifest())),
        "invalid_manifest"
    );

    let mut manifest = baseline.clone();
    let prototype = manifest["fixtures"][0].clone();
    let fixtures = manifest["fixtures"].as_array_mut().unwrap();
    while fixtures.len() <= 10_000 {
        let mut fixture = prototype.clone();
        fixture["id"] = json!(format!("count-probe-{}", fixtures.len()));
        fixtures.push(fixture);
    }
    write_json(&controls.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&controls.manifest())),
        "resource_exhausted"
    );

    let oversized_path = controls.0.join("inputs/oversized.json");
    fs::write(&oversized_path, vec![b' '; 16_777_217]).unwrap();
    let mut manifest = baseline.clone();
    manifest["fixtures"][0]["input"] = json!("inputs/oversized.json");
    write_json(&controls.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&controls.manifest())),
        "resource_exhausted"
    );

    let mut manifest = baseline.clone();
    manifest["package_schema"]["path"] = json!("/etc/shadow");
    write_json(&controls.manifest(), &manifest);
    let absolute = run_manifest(&controls.manifest());
    assert_eq!(error_code(&absolute), "unsafe_path");
    assert!(!String::from_utf8_lossy(&absolute.stderr).contains("/etc/shadow"));

    write_json(&controls.manifest(), &baseline);
    let bare = Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
        .current_dir(&controls.0)
        .args(["run", "--manifest", "manifest.json"])
        .output()
        .unwrap();
    assert!(
        bare.status.success(),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );

    let mut deeply_nested = vec![b'['; 2_048];
    deeply_nested.push(b'0');
    deeply_nested.extend(std::iter::repeat(b']').take(2_048));
    fs::write(controls.manifest(), &deeply_nested).unwrap();
    assert_eq!(
        error_code(&run_manifest(&controls.manifest())),
        "resource_exhausted"
    );

    write_json(&controls.manifest(), &baseline);
    let schema_path = controls
        .0
        .join("schemas/contract-conformance-manifest-v1.schema.json");
    fs::write(&schema_path, &deeply_nested).unwrap();
    let mut manifest = baseline;
    manifest["conformance_schema"]["sha256"] = json!(quire_contract_ir::hex_digest(&deeply_nested));
    write_json(&controls.manifest(), &manifest);
    assert_eq!(
        error_code(&run_manifest(&controls.manifest())),
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
    let linked_run =
        catch_unwind(|| quire_contract_ir::run_manifest(&corpus().join("manifest.json")));
    let linked_results = linked_run
        .expect("the linked runner panicked over the complete corpus")
        .expect("the linked runner rejected the published corpus");
    assert_eq!(linked_results.len(), 89);
    assert!(linked_results
        .iter()
        .all(|result| result.status() == quire_contract_ir::FixtureStatus::Match));

    let mut deeply_nested_package = String::from(
        r#"{"id":"agent-ix/depth","schema_version":{"major":1,"minor":1},"source":{"document":"depth","revision":1},"requirements":[],"ignored":"#,
    );
    deeply_nested_package.extend(std::iter::repeat('[').take(2_048));
    deeply_nested_package.push('0');
    deeply_nested_package.extend(std::iter::repeat(']').take(2_048));
    deeply_nested_package.push('}');
    let deep_decode = catch_unwind(|| {
        ContractPackage::from_json_str(&deeply_nested_package, ValidationOptions::strict())
    })
    .expect("deep package decoding panicked")
    .unwrap_err();
    assert_eq!(deep_decode[0].code, DiagnosticCode::SemanticInputTooLarge);

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

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            files.extend(walk_files(&entry.path()));
        } else {
            files.push(entry.path());
        }
    }
    files.sort();
    files
}

fn verify_sidecar(file: &Path, sidecar: &Path) {
    let sidecar = fs::read_to_string(sidecar).unwrap();
    let expected = sidecar.split_whitespace().next().unwrap();
    assert_eq!(
        quire_contract_ir::hex_digest(&fs::read(file).unwrap()),
        expected
    );
}
