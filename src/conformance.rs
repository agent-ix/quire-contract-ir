use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Component, Path},
};

use jsonschema::{Draft, JSONSchema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest as _, Sha256};

use crate::{
    classify_coverage, migrate_reference_body, ArtifactId, ArtifactTrace, CanonicalDigest,
    CanonicalOutput, CanonicalProfile, ClauseRef, ContractPackage, Diagnostic, DiagnosticCode,
    ReferenceBody, RequirementRef, SchemaVersion, SourceSpan,
};

pub const CONFORMANCE_PROTOCOL: &str = "quire.contract.conformance-jsonl/v1";
pub const PACKAGE_SCHEMA_ID: &str =
    "https://agent-ix.github.io/quire-contract-ir/schemas/contract-package-reference-v1.schema.json";
pub const CONFORMANCE_SCHEMA_ID: &str =
    "https://agent-ix.github.io/quire-contract-ir/schemas/contract-conformance-manifest-v1.schema.json";
pub const MAX_CONFORMANCE_FILE_BYTES: u64 = 16_777_216;
pub const MAX_CONFORMANCE_FIXTURES: u32 = 10_000;
pub const MAX_SEMANTIC_NODES: u32 = 25_000;
pub const MAX_SEMANTIC_DEPTH: u32 = 256;
pub const MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10_000;
pub(crate) const MAX_WIRE_JSON_DEPTH: u32 = MAX_SEMANTIC_DEPTH * 2 + 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationOptions {
    strict: bool,
}

impl ValidationOptions {
    pub const fn strict() -> Self {
        Self { strict: true }
    }

    pub const fn is_strict(self) -> bool {
        self.strict
    }
}

pub const PUBLIC_CONSTRUCT_TAGS: &[&str] = &[
    "artifact.depth.deep",
    "artifact.depth.shallow",
    "clause_kind.assertion",
    "clause_kind.case",
    "clause_kind.information",
    "clause_kind.invariant",
    "clause_kind.postcondition",
    "clause_kind.precondition",
    "coverage.class.deep",
    "coverage.class.orphaned",
    "coverage.class.shallow",
    "coverage.class.uncovered",
    "declaration.enum",
    "declaration.function",
    "declaration.input",
    "declaration.record",
    "declaration.state",
    "dependency.kind.enum_variant",
    "dependency.kind.field",
    "dependency.kind.input",
    "dependency.kind.pure_function",
    "dependency.kind.state",
    "execution.handler",
    "execution.initialization",
    "execution.post",
    "execution.pre",
    "expression.boolean",
    "expression.boolean_literal",
    "expression.boolean_not",
    "expression.call",
    "expression.collection_literal",
    "expression.compare",
    "expression.enum_literal",
    "expression.field_access",
    "expression.index",
    "expression.integer_literal",
    "expression.is_present",
    "expression.length",
    "expression.local_reference",
    "expression.numeric",
    "expression.numeric_negate",
    "expression.option_none",
    "expression.option_some",
    "expression.quantifier",
    "expression.rational_literal",
    "expression.record_literal",
    "expression.text_literal",
    "expression.unwrap",
    "expression.value_reference",
    "migration.reference_body_1_0_to_1_1",
    "reference_body.composite",
    "reference_body.literal",
    "reference_body.reference",
    "type.boolean",
    "type.collection",
    "type.enum",
    "type.integer",
    "type.option",
    "type.rational",
    "type.record",
    "type.text",
];

pub const CONFORMANCE_BOUNDARIES: &[&str] = &[
    "artifact.cross_package",
    "artifact.digest_mismatch",
    "artifact.duplicate",
    "artifact.missing",
    "artifact.stale",
    "canonical.escape_controls",
    "canonical.resource_failure",
    "canonical.semantic_set_order",
    "canonical.sequence_order",
    "collection.declared_maximum",
    "collection.declared_out_of_range",
    "collection.maximum",
    "collection.minimum",
    "collection.over_maximum",
    "expression.depth.maximum",
    "expression.depth.over_maximum",
    "expression.nodes.maximum",
    "expression.nodes.over_maximum",
    "integer.maximum",
    "integer.minimum",
    "integer.out_of_range",
    "rational.maximum_denominator",
    "rational.normalized",
    "rational.zero_denominator",
    "revision.current",
    "revision.stale",
    "schema.1_0",
    "schema.1_1",
    "schema.unknown_major",
    "schema.unregistered_minor",
    "schema.zero_major",
    "semantic.nodes.maximum",
    "semantic.nodes.over_maximum",
    "semantic_collection.maximum",
    "semantic_collection.over_maximum",
    "source_span.minimum",
    "source_span.reversed",
    "text.maximum",
    "text.over_maximum",
    "type.depth.maximum",
    "type.depth.over_maximum",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceOperation {
    Package,
    Expression,
    Migration,
    Coverage,
}

impl ConformanceOperation {
    pub const ALL: &'static [Self] = &[
        Self::Package,
        Self::Expression,
        Self::Migration,
        Self::Coverage,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Package => "package",
            Self::Expression => "expression",
            Self::Migration => "migration",
            Self::Coverage => "coverage",
        }
    }

    const fn input_definition(self) -> &'static str {
        match self {
            Self::Package => "packageInput",
            Self::Expression => "expressionInput",
            Self::Migration => "migrationInput",
            Self::Coverage => "coverageInput",
        }
    }

    const fn expectation_definition(self) -> &'static str {
        match self {
            Self::Package => "packageExpectation",
            Self::Expression => "expressionExpectation",
            Self::Migration => "migrationExpectation",
            Self::Coverage => "coverageExpectation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerErrorCode {
    InvalidInvocation,
    InvalidManifest,
    UnsupportedProfile,
    UnsafePath,
    FixtureIo,
    ResourceExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunnerError {
    protocol: &'static str,
    code: RunnerErrorCode,
    path: String,
    detail: &'static str,
}

impl RunnerError {
    pub fn new(code: RunnerErrorCode, path: impl Into<String>, detail: &'static str) -> Self {
        Self {
            protocol: CONFORMANCE_PROTOCOL,
            code,
            path: path.into(),
            detail,
        }
    }

    pub const fn code(&self) -> RunnerErrorCode {
        self.code
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DigestedPath {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    operation: ConformanceOperation,
    input: String,
    expectation: String,
    covers: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    corpus_id: String,
    package_schema: DigestedPath,
    conformance_schema: DigestedPath,
    inventory: DigestedPath,
    canonical_profile: String,
    protocol: String,
    fixtures: Vec<Fixture>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ToolIdentity {
    crate_version: &'static str,
    package_schema_path: String,
    package_schema_digest: String,
    canonical_profile: String,
    runner_protocol: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureStatus {
    Match,
    Mismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FixtureResult {
    protocol: &'static str,
    corpus_id: String,
    fixture_id: String,
    operation: ConformanceOperation,
    status: FixtureStatus,
    mismatch_kinds: Vec<&'static str>,
    actual: Value,
    tool: ToolIdentity,
}

impl FixtureResult {
    pub const fn status(&self) -> FixtureStatus {
        self.status
    }
}

struct LoadedFixture {
    fixture: Fixture,
    input: Value,
    expected: Value,
}

pub fn expected_inventory() -> Vec<String> {
    let mut inventory = Vec::new();
    inventory.extend(
        PUBLIC_CONSTRUCT_TAGS
            .iter()
            .map(|tag| format!("construct:{tag}")),
    );
    inventory.extend(
        DiagnosticCode::ALL
            .iter()
            .map(|code| format!("diagnostic:{}", code.as_str())),
    );
    inventory.extend([
        "obligation:option_presence".to_owned(),
        "obligation:non_zero_divisor".to_owned(),
        "obligation:index_in_bounds".to_owned(),
        "obligation:checked_range".to_owned(),
    ]);
    inventory.extend(
        CONFORMANCE_BOUNDARIES
            .iter()
            .map(|boundary| format!("boundary:{boundary}")),
    );
    inventory.extend(
        ConformanceOperation::ALL
            .iter()
            .map(|operation| format!("operation:{}", operation.as_str())),
    );
    inventory.sort();
    inventory
}

pub fn run_manifest(path: &Path) -> Result<Vec<FixtureResult>, RunnerError> {
    let manifest_bytes = read_manifest(path)?;
    let manifest_value: Value =
        parse_json(&manifest_bytes, "manifest", "manifest JSON is malformed")?;
    let root = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|_| RunnerError::new(RunnerErrorCode::FixtureIo, "manifest", "root unreadable"))?;
    let bootstrap_schema = manifest_value
        .get("conformance_schema")
        .and_then(Value::as_object)
        .and_then(|value| value.get("path"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            RunnerError::new(
                RunnerErrorCode::InvalidManifest,
                "conformance_schema.path",
                "manifest shape is invalid",
            )
        })?;
    let conformance_schema_bytes =
        read_relative(&root, bootstrap_schema, "conformance_schema.path")?;
    let conformance_schema: Value = parse_json(
        &conformance_schema_bytes,
        "conformance_schema",
        "schema JSON is malformed",
    )?;
    preflight_manifest_paths(&manifest_value)?;
    require_schema_identity(
        &conformance_schema,
        CONFORMANCE_SCHEMA_ID,
        "conformance_schema",
    )?;
    validate_complete_schema(&conformance_schema, "conformance_schema")?;
    validate_named(&conformance_schema, "manifest", &manifest_value)?;
    let manifest: Manifest = serde_json::from_value(manifest_value).map_err(|_| {
        RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            "manifest",
            "manifest shape is invalid",
        )
    })?;
    validate_profiles(&manifest)?;
    verify_digest(
        &conformance_schema_bytes,
        &manifest.conformance_schema.sha256,
        "conformance_schema.sha256",
    )?;
    let package_schema_bytes =
        read_relative(&root, &manifest.package_schema.path, "package_schema.path")?;
    verify_digest(
        &package_schema_bytes,
        &manifest.package_schema.sha256,
        "package_schema.sha256",
    )?;
    let package_schema: Value = parse_json(
        &package_schema_bytes,
        "package_schema",
        "schema JSON is malformed",
    )?;
    require_schema_identity(&package_schema, PACKAGE_SCHEMA_ID, "package_schema")?;
    validate_complete_schema(&package_schema, "package_schema")?;
    let inventory_bytes = read_relative(&root, &manifest.inventory.path, "inventory.path")?;
    verify_digest(
        &inventory_bytes,
        &manifest.inventory.sha256,
        "inventory.sha256",
    )?;
    let inventory: Vec<String> =
        parse_json(&inventory_bytes, "inventory", "inventory JSON is malformed")?;
    validate_inventory(&manifest, &inventory)?;

    let mut loaded = Vec::with_capacity(manifest.fixtures.len());
    for fixture in manifest.fixtures.iter().cloned() {
        let input_path = format!("fixtures.{}.input", fixture.id);
        let expectation_path = format!("fixtures.{}.expectation", fixture.id);
        let input_bytes = read_relative(&root, &fixture.input, &input_path)?;
        let expectation_bytes = read_relative(&root, &fixture.expectation, &expectation_path)?;
        let input: Value = parse_json(&input_bytes, input_path, "fixture JSON is malformed")?;
        let mut expected: Value = parse_json(
            &expectation_bytes,
            expectation_path,
            "expectation JSON is malformed",
        )?;
        validate_named(
            &conformance_schema,
            fixture.operation.input_definition(),
            &input,
        )?;
        validate_named(
            &conformance_schema,
            fixture.operation.expectation_definition(),
            &expected,
        )?;
        expand_canonical_paths(&root, &mut expected)?;
        loaded.push(LoadedFixture {
            fixture,
            input,
            expected,
        });
    }

    let tool = ToolIdentity {
        crate_version: env!("CARGO_PKG_VERSION"),
        package_schema_path: manifest.package_schema.path.clone(),
        package_schema_digest: manifest.package_schema.sha256.clone(),
        canonical_profile: manifest.canonical_profile.clone(),
        runner_protocol: CONFORMANCE_PROTOCOL,
    };
    loaded
        .into_iter()
        .map(|loaded| {
            let operation = loaded.fixture.operation;
            let execution_input = loaded.input.clone();
            let actual = std::thread::Builder::new()
                .name("quire-fixture-execution".to_owned())
                .stack_size(16 * 1024 * 1024)
                .spawn(move || execute(operation, execution_input))
                .map_err(|_| {
                    RunnerError::new(
                        RunnerErrorCode::ResourceExhausted,
                        "fixture_execution",
                        "fixture execution thread cannot be created",
                    )
                })?
                .join()
                .map_err(|_| {
                    RunnerError::new(
                        RunnerErrorCode::ResourceExhausted,
                        "fixture_execution",
                        "fixture execution thread terminated unexpectedly",
                    )
                })?;
            validate_successful_package_schema(
                operation,
                &loaded.input,
                &actual,
                &package_schema,
                &loaded.fixture.id,
            )?;
            validate_observed_coverage(&loaded.fixture, &loaded.input, &actual)?;
            let mismatch_kinds = mismatch_kinds(&actual, &loaded.expected);
            Ok(FixtureResult {
                protocol: CONFORMANCE_PROTOCOL,
                corpus_id: manifest.corpus_id.clone(),
                fixture_id: loaded.fixture.id,
                operation: loaded.fixture.operation,
                status: if mismatch_kinds.is_empty() {
                    FixtureStatus::Match
                } else {
                    FixtureStatus::Mismatch
                },
                mismatch_kinds,
                actual,
                tool: tool.clone(),
            })
        })
        .collect()
}

fn validate_successful_package_schema(
    operation: ConformanceOperation,
    input: &Value,
    actual: &Value,
    package_schema: &Value,
    fixture_id: &str,
) -> Result<(), RunnerError> {
    let succeeded = actual.get("valid").and_then(Value::as_bool) == Some(true)
        || (operation == ConformanceOperation::Coverage
            && actual.get("coverage").is_some_and(|value| !value.is_null()));
    if !succeeded {
        return Ok(());
    }
    let package = match operation {
        ConformanceOperation::Package => input.get("package").unwrap_or(input),
        ConformanceOperation::Migration | ConformanceOperation::Coverage => {
            input.get("package").ok_or_else(|| {
                RunnerError::new(
                    RunnerErrorCode::InvalidManifest,
                    format!("fixtures.{fixture_id}.input.package"),
                    "successful operation has no package input",
                )
            })?
        }
        ConformanceOperation::Expression => return Ok(()),
    };
    ensure_bounded_json_depth(package, "package")?;
    let schema = package_schema.clone();
    let package = package.clone();
    let path = format!("fixtures.{fixture_id}.input.package");
    std::thread::Builder::new()
        .name("quire-package-schema-validation".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&schema)
                .map_err(|_| {
                    RunnerError::new(
                        RunnerErrorCode::InvalidManifest,
                        "package_schema",
                        "schema cannot be compiled",
                    )
                })?;
            if compiled.is_valid(&package) {
                Ok(())
            } else {
                Err(RunnerError::new(
                    RunnerErrorCode::InvalidManifest,
                    path,
                    "successful package does not match published schema",
                ))
            }
        })
        .map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "package_schema_validation",
                "package schema validation thread cannot be created",
            )
        })?
        .join()
        .map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "package_schema_validation",
                "package schema validation thread terminated unexpectedly",
            )
        })?
}

fn parse_json<T: DeserializeOwned>(
    bytes: &[u8],
    path: impl Into<String>,
    malformed_detail: &'static str,
) -> Result<T, RunnerError> {
    let path = path.into();
    if json_nesting_exceeds(bytes, MAX_WIRE_JSON_DEPTH) {
        return Err(RunnerError::new(
            RunnerErrorCode::ResourceExhausted,
            path,
            "JSON nesting exceeds decode limit",
        ));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    deserializer.disable_recursion_limit();
    T::deserialize(serde_stacker::Deserializer::new(&mut deserializer))
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidManifest, path, malformed_detail))
}

pub(crate) fn json_nesting_exceeds(bytes: &[u8], maximum: u32) -> bool {
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    for byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match *byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
                if depth > maximum {
                    return true;
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

fn preflight_manifest_paths(manifest: &Value) -> Result<(), RunnerError> {
    let object = manifest.as_object().ok_or_else(|| {
        RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            "manifest",
            "manifest shape is invalid",
        )
    })?;
    for name in ["package_schema", "conformance_schema", "inventory"] {
        if let Some(path) = object
            .get(name)
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("path"))
            .and_then(Value::as_str)
        {
            if !safe_relative(path) {
                return Err(RunnerError::new(
                    RunnerErrorCode::UnsafePath,
                    format!("{name}.path"),
                    "path is unsafe",
                ));
            }
        }
    }
    if let Some(fixtures) = object.get("fixtures").and_then(Value::as_array) {
        if fixtures.len() > MAX_CONFORMANCE_FIXTURES as usize {
            return Err(RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "fixtures",
                "fixture count exceeds limit",
            ));
        }
        for (index, fixture) in fixtures.iter().enumerate() {
            for name in ["input", "expectation"] {
                if let Some(path) = fixture.get(name).and_then(Value::as_str) {
                    if !safe_relative(path) {
                        return Err(RunnerError::new(
                            RunnerErrorCode::UnsafePath,
                            format!("fixtures.{index}.{name}"),
                            "path is unsafe",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn read_manifest(path: &Path) -> Result<Vec<u8>, RunnerError> {
    let before = fs::metadata(path).map_err(|_| {
        RunnerError::new(
            RunnerErrorCode::FixtureIo,
            "manifest",
            "manifest unreadable",
        )
    })?;
    if !before.is_file() {
        return Err(RunnerError::new(
            RunnerErrorCode::FixtureIo,
            "manifest",
            "manifest is not a regular file",
        ));
    }
    if before.len() > MAX_CONFORMANCE_FILE_BYTES {
        return Err(RunnerError::new(
            RunnerErrorCode::ResourceExhausted,
            "manifest",
            "manifest exceeds byte limit",
        ));
    }
    let bytes = fs::read(path).map_err(|_| {
        RunnerError::new(
            RunnerErrorCode::FixtureIo,
            "manifest",
            "manifest unreadable",
        )
    })?;
    if bytes.len() as u64 > MAX_CONFORMANCE_FILE_BYTES {
        return Err(RunnerError::new(
            RunnerErrorCode::ResourceExhausted,
            "manifest",
            "manifest exceeds byte limit",
        ));
    }
    let after = fs::metadata(path).map_err(|_| {
        RunnerError::new(
            RunnerErrorCode::FixtureIo,
            "manifest",
            "manifest unreadable",
        )
    })?;
    if bytes.len() as u64 != before.len()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err(RunnerError::new(
            RunnerErrorCode::FixtureIo,
            "manifest",
            "manifest changed during preload",
        ));
    }
    Ok(bytes)
}

fn safe_relative(path: &str) -> bool {
    let path = Path::new(path);
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| {
            matches!(component, Component::Normal(_))
                && !component.as_os_str().is_empty()
                && component.as_os_str() != "."
                && component.as_os_str() != ".."
        })
}

fn read_relative(root: &Path, relative: &str, field: &str) -> Result<Vec<u8>, RunnerError> {
    if !safe_relative(relative) {
        return Err(RunnerError::new(
            RunnerErrorCode::UnsafePath,
            field,
            "path is unsafe",
        ));
    }
    let joined = root.join(relative);
    let resolved = joined
        .canonicalize()
        .map_err(|_| RunnerError::new(RunnerErrorCode::FixtureIo, field, "fixture unreadable"))?;
    if !resolved.starts_with(root) {
        return Err(RunnerError::new(
            RunnerErrorCode::UnsafePath,
            field,
            "path escapes corpus root",
        ));
    }
    let before = fs::metadata(&resolved)
        .map_err(|_| RunnerError::new(RunnerErrorCode::FixtureIo, field, "fixture unreadable"))?;
    if !before.is_file() {
        return Err(RunnerError::new(
            RunnerErrorCode::FixtureIo,
            field,
            "fixture is not a regular file",
        ));
    }
    if before.len() > MAX_CONFORMANCE_FILE_BYTES {
        return Err(RunnerError::new(
            RunnerErrorCode::ResourceExhausted,
            field,
            "fixture exceeds byte limit",
        ));
    }
    let bytes = fs::read(&resolved)
        .map_err(|_| RunnerError::new(RunnerErrorCode::FixtureIo, field, "fixture unreadable"))?;
    if bytes.len() as u64 > MAX_CONFORMANCE_FILE_BYTES {
        return Err(RunnerError::new(
            RunnerErrorCode::ResourceExhausted,
            field,
            "fixture exceeds byte limit",
        ));
    }
    let after = fs::metadata(&resolved)
        .map_err(|_| RunnerError::new(RunnerErrorCode::FixtureIo, field, "fixture unreadable"))?;
    if bytes.len() as u64 != before.len()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err(RunnerError::new(
            RunnerErrorCode::FixtureIo,
            field,
            "fixture changed during preload",
        ));
    }
    Ok(bytes)
}

fn validate_named(schema: &Value, definition: &str, instance: &Value) -> Result<(), RunnerError> {
    ensure_bounded_json_depth(instance, definition)?;
    let schema = schema.clone();
    let definition = definition.to_owned();
    let instance = instance.clone();
    std::thread::Builder::new()
        .name("quire-schema-validation".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || validate_named_on_bounded_stack(&schema, &definition, &instance))
        .map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "schema_validation",
                "schema validation thread cannot be created",
            )
        })?
        .join()
        .map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                "schema_validation",
                "schema validation thread terminated unexpectedly",
            )
        })?
}

fn validate_named_on_bounded_stack(
    schema: &Value,
    definition: &str,
    instance: &Value,
) -> Result<(), RunnerError> {
    let definitions = schema.get("definitions").cloned().ok_or_else(|| {
        RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            "conformance_schema",
            "schema definitions are absent",
        )
    })?;
    let wrapper = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "$ref": format!("#/definitions/{definition}"),
        "definitions": definitions,
    });
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&wrapper)
        .map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::InvalidManifest,
                "conformance_schema",
                "schema cannot be compiled",
            )
        })?;
    if compiled.is_valid(instance) {
        Ok(())
    } else {
        Err(RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            definition,
            "instance does not match schema",
        ))
    }
}

fn ensure_bounded_json_depth(value: &Value, path: &str) -> Result<(), RunnerError> {
    let mut pending = vec![(value, 1_u32)];
    while let Some((value, depth)) = pending.pop() {
        if depth > MAX_WIRE_JSON_DEPTH {
            return Err(RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                path.to_owned(),
                "JSON nesting exceeds validation limit",
            ));
        }
        match value {
            Value::Array(values) => {
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            Value::Object(values) => {
                pending.extend(values.values().map(|value| (value, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}

fn require_schema_identity(
    schema: &Value,
    expected: &str,
    path: &'static str,
) -> Result<(), RunnerError> {
    if schema.get("$id").and_then(Value::as_str) == Some(expected) {
        Ok(())
    } else {
        Err(RunnerError::new(
            RunnerErrorCode::UnsupportedProfile,
            path,
            "schema identity is unsupported",
        ))
    }
}

fn validate_complete_schema(schema: &Value, path: &'static str) -> Result<(), RunnerError> {
    ensure_bounded_json_depth(schema, path)?;
    let schema = schema.clone();
    std::thread::Builder::new()
        .name("quire-schema-compilation".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&schema)
                .map(|_| ())
                .map_err(|_| {
                    RunnerError::new(
                        RunnerErrorCode::InvalidManifest,
                        path,
                        "schema cannot be compiled",
                    )
                })
        })
        .map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                path,
                "schema compilation thread cannot be created",
            )
        })?
        .join()
        .map_err(|_| {
            RunnerError::new(
                RunnerErrorCode::ResourceExhausted,
                path,
                "schema compilation thread terminated unexpectedly",
            )
        })?
}

fn validate_profiles(manifest: &Manifest) -> Result<(), RunnerError> {
    if manifest.protocol != CONFORMANCE_PROTOCOL {
        return Err(RunnerError::new(
            RunnerErrorCode::UnsupportedProfile,
            "protocol",
            "runner protocol is unsupported",
        ));
    }
    if manifest.canonical_profile != crate::CANONICAL_PROFILE {
        return Err(RunnerError::new(
            RunnerErrorCode::UnsupportedProfile,
            "canonical_profile",
            "canonical profile is unsupported",
        ));
    }
    if manifest.fixtures.len() > MAX_CONFORMANCE_FIXTURES as usize {
        return Err(RunnerError::new(
            RunnerErrorCode::ResourceExhausted,
            "fixtures",
            "fixture count exceeds limit",
        ));
    }
    Ok(())
}

fn verify_digest(bytes: &[u8], expected: &str, path: &'static str) -> Result<(), RunnerError> {
    let actual = hex_digest(bytes);
    if actual == expected {
        Ok(())
    } else {
        Err(RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            path,
            "content digest mismatch",
        ))
    }
}

pub fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn validate_inventory(manifest: &Manifest, inventory: &[String]) -> Result<(), RunnerError> {
    if inventory != expected_inventory() {
        return Err(RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            "inventory",
            "inventory differs from public registries",
        ));
    }
    let mut seen_ids = HashSet::new();
    let mut covered = BTreeSet::new();
    for fixture in &manifest.fixtures {
        if !seen_ids.insert(&fixture.id) {
            return Err(RunnerError::new(
                RunnerErrorCode::InvalidManifest,
                "fixtures.id",
                "fixture ID is duplicated",
            ));
        }
        if fixture.covers.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(RunnerError::new(
                RunnerErrorCode::InvalidManifest,
                "fixtures.covers",
                "coverage tokens are not sorted and unique",
            ));
        }
        covered.extend(fixture.covers.iter().cloned());
    }
    if covered.into_iter().collect::<Vec<_>>() != inventory {
        return Err(RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            "fixtures.covers",
            "coverage union differs from inventory",
        ));
    }
    Ok(())
}

fn validate_observed_coverage(
    fixture: &Fixture,
    input: &Value,
    actual: &Value,
) -> Result<(), RunnerError> {
    let observed = observed_coverage(fixture.operation, input, actual);
    if let Some(token) = fixture
        .covers
        .iter()
        .find(|token| !observed.contains(token.as_str()))
    {
        return Err(RunnerError::new(
            RunnerErrorCode::InvalidManifest,
            format!("fixtures.{}.covers.{token}", fixture.id),
            "coverage token is not observed by the fixture",
        ));
    }
    Ok(())
}

fn observed_coverage(
    operation: ConformanceOperation,
    input: &Value,
    actual: &Value,
) -> BTreeSet<String> {
    let mut observed = BTreeSet::from([format!("operation:{}", operation.as_str())]);
    let succeeded = actual.get("valid").and_then(Value::as_bool) == Some(true)
        || (operation == ConformanceOperation::Coverage
            && actual.get("coverage").is_some_and(|value| !value.is_null()));
    let mut diagnostic_codes = BTreeSet::new();
    if let Some(diagnostics) = actual.get("diagnostics").and_then(Value::as_array) {
        for diagnostic in diagnostics {
            if let Some(code) = diagnostic.get("code").and_then(Value::as_str) {
                diagnostic_codes.insert(code);
                observed.insert(format!("diagnostic:{code}"));
                observe_diagnostic_boundaries(code, &mut observed);
            }
            if let Some(kind) = diagnostic.get("obligation_kind").and_then(Value::as_str) {
                observed.insert(format!("obligation:{kind}"));
            }
        }
    }
    observe_structural_boundaries(input, actual, succeeded, &diagnostic_codes, &mut observed);
    if succeeded {
        observe_constructs(input, actual, operation, &mut observed);
    }
    observed
}

fn observe_diagnostic_boundaries(code: &str, observed: &mut BTreeSet<String>) {
    let boundaries: &[&str] = match code {
        "canonicalization_resource_exhausted" => &["canonical.resource_failure"],
        "cross_package_reference" => &["artifact.cross_package"],
        "duplicate_artifact_trace" => &["artifact.duplicate"],
        "orphaned_requirement_reference" => &["artifact.missing"],
        "stale_requirement_revision" => &["artifact.stale", "revision.stale"],
        "stale_trace_digest" => &["artifact.digest_mismatch"],
        _ => &[],
    };
    observed.extend(
        boundaries
            .iter()
            .map(|boundary| format!("boundary:{boundary}")),
    );
}

fn observe_structural_boundaries(
    input: &Value,
    actual: &Value,
    succeeded: bool,
    diagnostic_codes: &BTreeSet<&str>,
    output: &mut BTreeSet<String>,
) {
    let mut observed = BTreeSet::new();
    let mut pending = vec![(input, 1_u32)];
    let mut semantic_nodes = 0_u32;
    while let Some((value, depth)) = pending.pop() {
        match value {
            Value::Array(values) => {
                if values.len() == MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                    observed.insert("boundary:semantic_collection.maximum".to_owned());
                    observed.insert("boundary:collection.maximum".to_owned());
                }
                if values.len() > MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                    observed.insert("boundary:semantic_collection.over_maximum".to_owned());
                    observed.insert("boundary:collection.over_maximum".to_owned());
                }
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            Value::Object(object) => {
                if let (Some(start), Some(end)) = (object.get("start"), object.get("end")) {
                    let start_offset = start.get("byte_offset").and_then(Value::as_u64);
                    let end_offset = end.get("byte_offset").and_then(Value::as_u64);
                    if start_offset
                        .zip(end_offset)
                        .is_some_and(|(start, end)| start > end)
                    {
                        observed.insert("boundary:source_span.reversed".to_owned());
                    }
                }
                if let Some(version) = object.get("schema_version").or_else(|| {
                    (object.contains_key("major") && object.contains_key("minor")).then_some(value)
                }) {
                    match (
                        version.get("major").and_then(Value::as_u64),
                        version.get("minor").and_then(Value::as_u64),
                    ) {
                        (Some(1), Some(0)) => {
                            observed.insert("boundary:schema.1_0".to_owned());
                        }
                        (Some(1), Some(1)) => {
                            observed.insert("boundary:schema.1_1".to_owned());
                        }
                        (Some(0), _) => {
                            observed.insert("boundary:schema.zero_major".to_owned());
                        }
                        (Some(1), Some(_)) => {
                            observed.insert("boundary:schema.unregistered_minor".to_owned());
                        }
                        (Some(_), _) => {
                            observed.insert("boundary:schema.unknown_major".to_owned());
                        }
                        _ => {}
                    }
                }
                if object.get("line").and_then(Value::as_u64) == Some(1)
                    && object.get("column").and_then(Value::as_u64) == Some(1)
                    && object.get("byte_offset").and_then(Value::as_u64) == Some(0)
                {
                    observed.insert("boundary:source_span.minimum".to_owned());
                }
                if let Some(kind) = object.get("kind").and_then(Value::as_str) {
                    if kind == "collection" {
                        match object.get("maximum_items").and_then(Value::as_u64) {
                            Some(0) => {
                                observed.insert("boundary:collection.minimum".to_owned());
                            }
                            Some(1) => {
                                observed.insert("boundary:collection.minimum".to_owned());
                            }
                            Some(value) if value == u32::MAX as u64 => {
                                observed.insert("boundary:collection.declared_maximum".to_owned());
                            }
                            Some(value) if value > u32::MAX as u64 => {
                                observed
                                    .insert("boundary:collection.declared_out_of_range".to_owned());
                            }
                            _ => {}
                        }
                    }
                    if kind == "integer" {
                        if object.get("minimum").and_then(Value::as_i64) == Some(i64::MIN) {
                            observed.insert("boundary:integer.minimum".to_owned());
                        }
                        if object.get("maximum").and_then(Value::as_i64) == Some(i64::MAX) {
                            observed.insert("boundary:integer.maximum".to_owned());
                        }
                        let minimum = object.get("minimum").and_then(Value::as_i64);
                        let maximum = object.get("maximum").and_then(Value::as_i64);
                        if minimum.zip(maximum).is_some_and(|(minimum, maximum)| {
                            minimum > maximum
                                || (object.get("domain").and_then(Value::as_str)
                                    == Some("unsigned")
                                    && minimum < 0)
                        }) {
                            observed.insert("boundary:integer.out_of_range".to_owned());
                        }
                    }
                    if kind == "rational"
                        && object.get("maximum_denominator").and_then(Value::as_u64)
                            == Some(i64::MAX as u64)
                    {
                        observed.insert("boundary:rational.maximum_denominator".to_owned());
                    }
                    if kind == "rational"
                        && object.get("maximum_denominator").and_then(Value::as_u64) == Some(0)
                    {
                        observed.insert("boundary:rational.zero_denominator".to_owned());
                    }
                }
                if object.contains_key("node") {
                    semantic_nodes = semantic_nodes.saturating_add(1);
                    if let Some(text) = object.get("value").and_then(Value::as_str) {
                        let length = text.chars().count();
                        if length == crate::MAX_TEXT_LENGTH as usize {
                            observed.insert("boundary:text.maximum".to_owned());
                        }
                        if length > crate::MAX_TEXT_LENGTH as usize {
                            observed.insert("boundary:text.over_maximum".to_owned());
                        }
                        if text.chars().any(char::is_control) {
                            observed.insert("boundary:canonical.escape_controls".to_owned());
                        }
                    }
                    if object.get("node").and_then(Value::as_str) == Some("rational_literal") {
                        if let (Some(numerator), Some(denominator)) = (
                            object.get("numerator").and_then(Value::as_i64),
                            object.get("denominator").and_then(Value::as_i64),
                        ) {
                            if denominator != 0 && numerator % denominator == 0 {
                                observed.insert("boundary:rational.normalized".to_owned());
                            }
                            if denominator == 0 {
                                observed.insert("boundary:rational.zero_denominator".to_owned());
                            }
                        }
                    }
                    if object.get("node").and_then(Value::as_str) == Some("collection_literal") {
                        let items = object.get("items").and_then(Value::as_array);
                        let maximum = object
                            .get("value_type")
                            .and_then(|value| value.get("maximum_items"))
                            .and_then(Value::as_u64);
                        if items
                            .zip(maximum)
                            .is_some_and(|(items, maximum)| items.len() as u64 > maximum)
                        {
                            observed.insert("boundary:collection.declared_out_of_range".to_owned());
                        }
                    }
                }
                pending.extend(object.values().map(|value| (value, depth + 1)));
            }
            _ => {}
        }
    }
    if semantic_nodes == crate::MAX_EXPRESSION_NODES {
        observed.insert("boundary:expression.nodes.maximum".to_owned());
    }
    if semantic_nodes > crate::MAX_EXPRESSION_NODES {
        observed.insert("boundary:expression.nodes.over_maximum".to_owned());
    }
    let expression_depth = expression_depth(input);
    if expression_depth == MAX_SEMANTIC_DEPTH {
        observed.insert("boundary:expression.depth.maximum".to_owned());
    }
    if expression_depth > MAX_SEMANTIC_DEPTH {
        observed.insert("boundary:expression.depth.over_maximum".to_owned());
    }
    let type_depth = type_depth(input);
    if type_depth == MAX_SEMANTIC_DEPTH {
        observed.insert("boundary:type.depth.maximum".to_owned());
    }
    if type_depth > MAX_SEMANTIC_DEPTH {
        observed.insert("boundary:type.depth.over_maximum".to_owned());
    }
    match expression_semantic_nodes(input) {
        Some(nodes) if nodes == MAX_SEMANTIC_NODES => {
            observed.insert("boundary:semantic.nodes.maximum".to_owned());
        }
        Some(nodes) if nodes > MAX_SEMANTIC_NODES => {
            observed.insert("boundary:semantic.nodes.over_maximum".to_owned());
        }
        _ => {}
    }
    if actual
        .get("canonical")
        .and_then(Value::as_array)
        .is_some_and(|values| !values.is_empty())
        && has_authored_set_out_of_order(input)
    {
        observed.insert("boundary:canonical.semantic_set_order".to_owned());
    }
    if actual
        .get("canonical")
        .and_then(Value::as_array)
        .is_some_and(|values| !values.is_empty())
        && has_authored_sequence(input)
    {
        observed.insert("boundary:canonical.sequence_order".to_owned());
    }
    observe_revisions(input, &mut observed);
    output.extend(
        observed
            .into_iter()
            .filter(|token| structural_boundary_observed(token, succeeded, diagnostic_codes)),
    );
}

fn structural_boundary_observed(
    token: &str,
    succeeded: bool,
    diagnostics: &BTreeSet<&str>,
) -> bool {
    let required_diagnostic = match token {
        "boundary:collection.minimum" => Some("unbounded_collection"),
        "boundary:collection.declared_out_of_range"
        | "boundary:integer.out_of_range"
        | "boundary:rational.zero_denominator" => Some("invalid_numeric_bounds"),
        "boundary:text.over_maximum" => Some("text_bound_exceeded"),
        "boundary:expression.nodes.over_maximum" => Some("expression_too_large"),
        "boundary:collection.over_maximum"
        | "boundary:expression.depth.over_maximum"
        | "boundary:semantic.nodes.over_maximum"
        | "boundary:semantic_collection.over_maximum"
        | "boundary:type.depth.over_maximum" => Some("semantic_input_too_large"),
        "boundary:revision.stale" => Some("stale_requirement_revision"),
        "boundary:schema.zero_major" => Some("invalid_schema_version"),
        "boundary:schema.unregistered_minor" => Some("unregistered_migration"),
        "boundary:schema.unknown_major" => Some("unsupported_schema_version"),
        "boundary:source_span.reversed" => Some("invalid_source_span"),
        _ => None,
    };
    required_diagnostic.map_or(succeeded, |code| diagnostics.contains(code))
}

fn has_authored_set_out_of_order(input: &Value) -> bool {
    let mut pending = vec![input];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(object) => {
                for key in ["requirements", "clauses", "types", "values", "functions"] {
                    if let Some(values) = object.get(key).and_then(Value::as_array) {
                        let names = values
                            .iter()
                            .filter_map(|value| {
                                value
                                    .get("id")
                                    .or_else(|| value.get("name"))
                                    .and_then(Value::as_str)
                            })
                            .collect::<Vec<_>>();
                        if names.windows(2).any(|pair| pair[0] > pair[1]) {
                            return true;
                        }
                    }
                }
                pending.extend(object.values());
            }
            _ => {}
        }
    }
    false
}

fn has_authored_sequence(input: &Value) -> bool {
    let mut pending = vec![input];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(object) => {
                if object
                    .get("children")
                    .and_then(Value::as_array)
                    .is_some_and(|values| values.len() > 1)
                    || object
                        .get("items")
                        .and_then(Value::as_array)
                        .is_some_and(|values| values.len() > 1)
                {
                    return true;
                }
                pending.extend(object.values());
            }
            _ => {}
        }
    }
    false
}

fn expression_depth(input: &Value) -> u32 {
    let Some(root) = input.get("expression") else {
        return 0;
    };
    let mut maximum = 0;
    let mut pending = vec![(root, 1_u32)];
    while let Some((value, depth)) = pending.pop() {
        if value.get("node").is_none() {
            continue;
        }
        maximum = maximum.max(depth);
        if let Some(object) = value.as_object() {
            for child in object.values() {
                match child {
                    Value::Object(_) if child.get("node").is_some() => {
                        pending.push((child, depth + 1));
                    }
                    Value::Array(values) => pending.extend(
                        values
                            .iter()
                            .filter(|value| value.get("node").is_some())
                            .map(|value| (value, depth + 1)),
                    ),
                    _ => {}
                }
            }
        }
    }
    maximum
}

fn type_depth(input: &Value) -> u32 {
    let mut maximum = 0;
    let mut pending = Vec::new();
    collect_type_roots(input, &mut pending);
    while let Some((value, depth)) = pending.pop() {
        maximum = maximum.max(depth);
        if let Some(child) = value.get("value").or_else(|| value.get("element")) {
            if child.get("kind").is_some() {
                pending.push((child, depth + 1));
            }
        }
    }
    maximum
}

fn collect_type_roots<'a>(input: &'a Value, output: &mut Vec<(&'a Value, u32)>) {
    if let Some(value) = input.get("expected_type") {
        output.push((value, 1));
    }
    for declaration in input
        .get("values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(value) = declaration.get("value_type") {
            output.push((value, 1));
        }
    }
    for declaration in input
        .get("types")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for field in declaration
            .get("fields")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(value) = field.get("value_type") {
                output.push((value, 1));
            }
        }
    }
    for declaration in input
        .get("functions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(value) = declaration.get("result_type") {
            output.push((value, 1));
        }
        for parameter in declaration
            .get("parameters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(value) = parameter.get("value_type") {
                output.push((value, 1));
            }
        }
    }
    if let Some(expression) = input.get("expression") {
        let mut pending = vec![expression];
        while let Some(value) = pending.pop() {
            if value.get("node").is_none() {
                continue;
            }
            if let Some(value_type) = value.get("value_type") {
                output.push((value_type, 1));
            }
            if let Some(object) = value.as_object() {
                for child in object.values() {
                    match child {
                        Value::Object(_) if child.get("node").is_some() => pending.push(child),
                        Value::Array(values) => pending
                            .extend(values.iter().filter(|value| value.get("node").is_some())),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn expression_semantic_nodes(input: &Value) -> Option<u32> {
    let mut nodes = count_expression_nodes(input.get("expression")?);
    let types = input.get("types")?.as_array()?;
    let values = input.get("values")?.as_array()?;
    let functions = input.get("functions")?.as_array()?;
    for declaration in types {
        nodes = nodes.saturating_add(1);
        nodes = nodes.saturating_add(
            declaration
                .get("variants")
                .or_else(|| declaration.get("fields"))
                .and_then(Value::as_array)
                .map_or(0, |values| values.len() as u32),
        );
    }
    nodes = nodes.saturating_add(values.len() as u32);
    nodes = nodes.saturating_add(
        functions
            .iter()
            .map(|function| {
                1 + function
                    .get("parameters")
                    .and_then(Value::as_array)
                    .map_or(0, |values| values.len() as u32)
            })
            .sum::<u32>(),
    );
    let mut roots = Vec::new();
    collect_type_roots(input, &mut roots);
    for (root, _) in roots {
        nodes = nodes.saturating_add(count_type_nodes(root));
    }
    Some(nodes)
}

fn count_expression_nodes(root: &Value) -> u32 {
    let mut nodes = 0_u32;
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        if value.get("node").is_none() {
            continue;
        }
        nodes = nodes.saturating_add(1);
        if let Some(object) = value.as_object() {
            for child in object.values() {
                match child {
                    Value::Object(_) if child.get("node").is_some() => pending.push(child),
                    Value::Array(values) => {
                        pending.extend(values.iter().filter(|value| value.get("node").is_some()))
                    }
                    _ => {}
                }
            }
        }
    }
    nodes
}

fn count_type_nodes(root: &Value) -> u32 {
    let mut nodes = 0_u32;
    let mut current = Some(root);
    while let Some(value) = current {
        nodes = nodes.saturating_add(1);
        current = value
            .get("value")
            .or_else(|| value.get("element"))
            .filter(|value| value.get("kind").is_some());
    }
    nodes
}

fn observe_revisions(input: &Value, observed: &mut BTreeSet<String>) {
    let Some(package) = input
        .get("package")
        .or_else(|| input.get("package").and_then(|value| value.get("package")))
        .or_else(|| input.get("id").is_some().then_some(input))
    else {
        return;
    };
    let Some(requirements) = package.get("requirements").and_then(Value::as_array) else {
        return;
    };
    if requirements
        .iter()
        .any(|requirement| requirement.get("revision").and_then(Value::as_u64) == Some(1))
    {
        observed.insert("boundary:revision.current".to_owned());
    }
}

fn observe_constructs(
    input: &Value,
    actual: &Value,
    operation: ConformanceOperation,
    observed: &mut BTreeSet<String>,
) {
    if operation == ConformanceOperation::Migration
        && actual
            .get("migration_receipt")
            .is_some_and(|value| !value.is_null())
    {
        observed.insert("construct:migration.reference_body_1_0_to_1_1".to_owned());
    }
    let expression_nodes = [
        "boolean_literal",
        "integer_literal",
        "rational_literal",
        "text_literal",
        "enum_literal",
        "option_none",
        "option_some",
        "record_literal",
        "collection_literal",
        "value_reference",
        "local_reference",
        "field_access",
        "is_present",
        "unwrap",
        "length",
        "index",
        "call",
        "numeric",
        "numeric_negate",
        "compare",
        "boolean_not",
        "boolean",
        "quantifier",
    ];
    let type_kinds = [
        "boolean",
        "integer",
        "rational",
        "text",
        "enum",
        "record",
        "option",
        "collection",
    ];
    let clause_kinds = [
        "precondition",
        "postcondition",
        "invariant",
        "assertion",
        "case",
        "information",
    ];
    let execution_kinds = ["pre", "post", "initialization", "handler"];
    let mut pending = vec![input, actual];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(object) => {
                if let Some(node) = object.get("node").and_then(Value::as_str) {
                    if expression_nodes.contains(&node) {
                        observed.insert(format!("construct:expression.{node}"));
                    } else if matches!(node, "literal" | "reference" | "composite") {
                        observed.insert(format!("construct:reference_body.{node}"));
                    }
                }
                if let Some(kind) = object.get("kind").and_then(Value::as_str) {
                    if type_kinds.contains(&kind) {
                        observed.insert(format!("construct:type.{kind}"));
                    }
                    if clause_kinds.contains(&kind) && object.contains_key("body") {
                        observed.insert(format!("construct:clause_kind.{kind}"));
                    }
                    if execution_kinds.contains(&kind)
                        && (object.contains_key("operation") || object.contains_key("name"))
                    {
                        observed.insert(format!("construct:execution.{kind}"));
                    }
                    if matches!(kind, "shallow" | "deep")
                        && (object.len() == 1 || object.contains_key("requirement_digest"))
                    {
                        observed.insert(format!("construct:artifact.depth.{kind}"));
                    }
                }
                pending.extend(object.values());
            }
            _ => {}
        }
    }
    if let Some(types) = input.get("types").and_then(Value::as_array) {
        for declaration in types {
            if let Some(kind) = declaration.get("kind").and_then(Value::as_str) {
                observed.insert(format!("construct:declaration.{kind}"));
            }
        }
    }
    if let Some(values) = input.get("values").and_then(Value::as_array) {
        for declaration in values {
            if let Some(kind) = declaration.get("kind").and_then(Value::as_str) {
                observed.insert(format!("construct:declaration.{kind}"));
            }
        }
    }
    if input
        .get("functions")
        .and_then(Value::as_array)
        .is_some_and(|values| !values.is_empty())
    {
        observed.insert("construct:declaration.function".to_owned());
    }
    if let Some(dependencies) = actual.get("dependencies").and_then(Value::as_array) {
        for dependency in dependencies {
            if let Some(kind) = dependency.get("kind").and_then(Value::as_str) {
                observed.insert(format!("construct:dependency.kind.{kind}"));
            }
        }
    }
    if let Some(coverage) = actual.get("coverage") {
        for collection in ["requirements", "artifacts"] {
            if let Some(rows) = coverage.get(collection).and_then(Value::as_array) {
                for row in rows {
                    if let Some(class) = row.get("class").and_then(Value::as_str) {
                        observed.insert(format!("construct:coverage.class.{class}"));
                    }
                }
            }
        }
    }
}

fn expand_canonical_paths(root: &Path, value: &mut Value) -> Result<(), RunnerError> {
    match value {
        Value::Array(values) => {
            for value in values {
                expand_canonical_paths(root, value)?;
            }
        }
        Value::Object(object) => {
            if let Some(path) = object.remove("bytes_path") {
                let path = path.as_str().ok_or_else(|| {
                    RunnerError::new(
                        RunnerErrorCode::InvalidManifest,
                        "bytes_path",
                        "canonical path is malformed",
                    )
                })?;
                let bytes = read_relative(root, path, "canonical.bytes_path")?;
                let text = String::from_utf8(bytes).map_err(|_| {
                    RunnerError::new(
                        RunnerErrorCode::InvalidManifest,
                        "canonical.bytes_path",
                        "canonical bytes are not UTF-8",
                    )
                })?;
                object.insert("bytes".to_owned(), Value::String(text));
            }
            for value in object.values_mut() {
                expand_canonical_paths(root, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn execute(operation: ConformanceOperation, input: Value) -> Value {
    match operation {
        ConformanceOperation::Package => execute_package(input),
        ConformanceOperation::Expression => crate::wire::execute_expression(input),
        ConformanceOperation::Migration => execute_migration(input),
        ConformanceOperation::Coverage => execute_coverage(input),
    }
}

fn execute_package(input: Value) -> Value {
    let (package_input, clause_resolutions, maximum_bytes) = if input.get("package").is_some() {
        let request: PackageProbeInput = match serde_json::from_value(input) {
            Ok(request) => request,
            Err(_) => return invalid_wire_actual("package"),
        };
        (
            request.package,
            request.clause_resolutions,
            request.canonical_maximum_bytes.unwrap_or(u64::MAX),
        )
    } else {
        (input, Vec::new(), u64::MAX)
    };
    match parse_package(&package_input) {
        Ok(package) => {
            let diagnostics = clause_resolutions
                .iter()
                .filter_map(|reference| package.resolve_clause(reference, None).err())
                .collect::<Vec<_>>();
            if !diagnostics.is_empty() {
                return invalid_actual(diagnostics);
            }
            match package_actual(&package, maximum_bytes) {
                Ok(value) => value,
                Err(diagnostic) => invalid_actual(vec![diagnostic]),
            }
        }
        Err(diagnostics) => invalid_actual(diagnostics),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageProbeInput {
    package: Value,
    #[serde(default)]
    clause_resolutions: Vec<ClauseRef>,
    #[serde(default)]
    canonical_maximum_bytes: Option<u64>,
}

fn parse_package(input: &Value) -> Result<ContractPackage<ReferenceBody>, Vec<Diagnostic>> {
    serde_json::to_string(input)
        .map_err(|_| {
            vec![Diagnostic::error(
                DiagnosticCode::InvalidWireFormat,
                "package cannot be encoded",
                "document",
            )]
        })
        .and_then(|value| ContractPackage::from_json_str(&value, ValidationOptions::strict()))
}

fn package_actual(
    package: &ContractPackage<ReferenceBody>,
    maximum_bytes: u64,
) -> Result<Value, Diagnostic> {
    let mut canonical = vec![canonical_value(
        "package",
        package.canonical_package_with_limit(CanonicalProfile::V1, maximum_bytes)?,
    )?];
    let mut requirements = package.requirements().iter().collect::<Vec<_>>();
    requirements.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
    let mut dependencies = BTreeSet::new();
    for requirement in requirements {
        canonical.push(canonical_value(
            &format!("requirement:{}", requirement.id().as_str()),
            package.canonical_requirement_with_limit(
                requirement,
                CanonicalProfile::V1,
                maximum_bytes,
            )?,
        )?);
        let mut clauses = requirement.clauses().iter().collect::<Vec<_>>();
        clauses.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
        for clause in clauses {
            canonical.push(canonical_value(
                &format!(
                    "clause:{}:{}",
                    requirement.id().as_str(),
                    clause.id().as_str()
                ),
                package.canonical_clause_with_limit(
                    requirement,
                    clause,
                    CanonicalProfile::V1,
                    maximum_bytes,
                )?,
            )?);
            dependencies.extend(clause.dependencies());
        }
    }
    Ok(json!({
        "valid": true,
        "diagnostics": [],
        "canonical": canonical,
        "dependencies": dependencies,
    }))
}

pub(crate) fn canonical_value(
    identity: &str,
    output: CanonicalOutput,
) -> Result<Value, Diagnostic> {
    let bytes = std::str::from_utf8(output.bytes().as_slice()).map_err(|_| {
        Diagnostic::error(
            DiagnosticCode::InvalidWireFormat,
            "canonical output is not UTF-8",
            "canonical",
        )
    })?;
    Ok(json!({
        "identity": identity,
        "kind": output.kind().as_str(),
        "bytes": bytes,
        "digest": output.digest().to_string(),
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MigrationInput {
    package: Value,
    target_version: SchemaVersion,
}

fn execute_migration(input: Value) -> Value {
    let request: MigrationInput = match serde_json::from_value(input) {
        Ok(request) => request,
        Err(_) => return invalid_wire_actual("migration"),
    };
    let package = match parse_package(&request.package) {
        Ok(package) => package,
        Err(diagnostics) => return migration_invalid_actual(diagnostics),
    };
    match migrate_reference_body(package, request.target_version, CanonicalProfile::V1) {
        Ok((package, receipt)) => match package.canonical_package(CanonicalProfile::V1) {
            Ok(output) => match canonical_value("migrated_package", output) {
                Ok(output) => json!({
                    "valid": true,
                    "diagnostics": [],
                    "canonical": [output],
                    "migration_receipt": receipt,
                }),
                Err(diagnostic) => migration_invalid_actual(vec![diagnostic]),
            },
            Err(diagnostic) => migration_invalid_actual(vec![diagnostic]),
        },
        Err(diagnostics) => migration_invalid_actual(diagnostics),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageInput {
    package: Value,
    traces: Vec<WireArtifactTrace>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireArtifactTrace {
    artifact_id: String,
    source: SourceSpan,
    target: RequirementRef,
    target_span: SourceSpan,
    depth: WireTraceDepth,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum WireTraceDepth {
    Shallow,
    Deep {
        requirement_digest: CanonicalDigest,
        digest_span: SourceSpan,
    },
}

impl WireArtifactTrace {
    fn validate(self) -> Result<ArtifactTrace, Diagnostic> {
        let artifact_id = ArtifactId::new(self.artifact_id)?;
        Ok(match self.depth {
            WireTraceDepth::Shallow => {
                ArtifactTrace::shallow(artifact_id, self.source, self.target, self.target_span)
            }
            WireTraceDepth::Deep {
                requirement_digest,
                digest_span,
            } => ArtifactTrace::deep(
                artifact_id,
                self.source,
                self.target,
                self.target_span,
                requirement_digest,
                digest_span,
            ),
        })
    }
}

fn execute_coverage(input: Value) -> Value {
    let request: CoverageInput = match serde_json::from_value(input) {
        Ok(request) => request,
        Err(_) => return invalid_wire_actual("coverage"),
    };
    let package = match parse_package(&request.package) {
        Ok(package) => package,
        Err(diagnostics) => return coverage_invalid_actual(diagnostics),
    };
    let traces = match request
        .traces
        .into_iter()
        .map(WireArtifactTrace::validate)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(traces) => traces,
        Err(diagnostic) => return coverage_invalid_actual(vec![diagnostic]),
    };
    match classify_coverage(&package, &traces, CanonicalProfile::V1) {
        Ok(result) => json!({
            "diagnostics": diagnostics_value(result.diagnostics()),
            "coverage": result.report(),
        }),
        Err(diagnostic) => coverage_invalid_actual(vec![diagnostic]),
    }
}

fn invalid_wire_actual(path: &'static str) -> Value {
    invalid_actual(vec![Diagnostic::error(
        DiagnosticCode::InvalidWireFormat,
        "operation input cannot be decoded",
        path,
    )])
}

fn invalid_actual(diagnostics: Vec<Diagnostic>) -> Value {
    json!({
        "valid": false,
        "diagnostics": diagnostics_value(&diagnostics),
        "canonical": [],
        "dependencies": [],
    })
}

fn migration_invalid_actual(diagnostics: Vec<Diagnostic>) -> Value {
    json!({
        "valid": false,
        "diagnostics": diagnostics_value(&diagnostics),
        "canonical": [],
        "migration_receipt": null,
    })
}

fn coverage_invalid_actual(diagnostics: Vec<Diagnostic>) -> Value {
    json!({
        "diagnostics": diagnostics_value(&diagnostics),
        "coverage": null,
    })
}

pub(crate) fn diagnostics_value(diagnostics: &[Diagnostic]) -> Value {
    Value::Array(
        diagnostics
            .iter()
            .map(|diagnostic| {
                let mut value = Map::new();
                value.insert("code".to_owned(), json!(diagnostic.code));
                value.insert("severity".to_owned(), json!(diagnostic.severity));
                value.insert("path".to_owned(), json!(diagnostic.path));
                if let Some(span) = &diagnostic.span {
                    value.insert("span".to_owned(), json!(span));
                }
                if !diagnostic.related.is_empty() {
                    value.insert("related".to_owned(), json!(diagnostic.related));
                }
                if let Some(kind) = diagnostic.obligation_kind {
                    value.insert("obligation_kind".to_owned(), json!(kind));
                }
                Value::Object(value)
            })
            .collect(),
    )
}

fn mismatch_kinds(actual: &Value, expected: &Value) -> Vec<&'static str> {
    let mut kinds = Vec::new();
    if field(actual, "valid") != field(expected, "valid") {
        kinds.push("validity");
    }
    if field(actual, "diagnostics") != field(expected, "diagnostics") {
        kinds.push("diagnostics");
    }
    if canonical_projection(actual, "bytes") != canonical_projection(expected, "bytes") {
        kinds.push("canonical_bytes");
    }
    if canonical_projection(actual, "digest") != canonical_projection(expected, "digest") {
        kinds.push("canonical_digest");
    }
    if field(actual, "dependencies") != field(expected, "dependencies") {
        kinds.push("dependencies");
    }
    if field(actual, "migration_receipt") != field(expected, "migration_receipt") {
        kinds.push("migration_receipt");
    }
    if field(actual, "coverage") != field(expected, "coverage") {
        kinds.push("coverage");
    }
    kinds
}

fn field<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    value.as_object().and_then(|value| value.get(name))
}

fn canonical_projection(value: &Value, selected: &str) -> Value {
    let Some(values) = field(value, "canonical").and_then(Value::as_array) else {
        return Value::Null;
    };
    Value::Array(
        values
            .iter()
            .map(|value| {
                json!({
                    "identity": field(value, "identity"),
                    "kind": field(value, "kind"),
                    selected: field(value, selected),
                })
            })
            .collect(),
    )
}
