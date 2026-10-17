//! NFR-003 (TC-019): every declared invalid-input class fails closed with a
//! stable diagnostic code, no public panic, and no canonical identity or
//! covered status.
//!
//! The declared classes are the semantic codes STD-001 registers for FR-011
//! through FR-019 (`DiagnosticCode`). Each class is driven through the public
//! API on a committed corpus fixture: the conformance runner (`run_corpus`,
//! whose fixture execution reports a panic as a runner error, so a panicking
//! class fails the `unwrap`) and, for wire-level classes, `ContractPackage`
//! decoding directly.

use std::{collections::BTreeSet, fs, path::PathBuf};

use ix_trace_rs::trace;
use quire_contract_model::{
    run_corpus, ContractPackage, DiagnosticCode, FixtureResult, ValidationOptions,
};
use serde_json::Value;

/// One invalid-input class and the fixtures that exercise it.
struct Class {
    code: &'static str,
    fixtures: &'static [&'static str],
}

/// The declared invalid-input classes, written out from STD-001's code tables,
/// each with its corpus fixtures. `declared_classes_match_std001` ties this
/// list to both the STD-001 text and `DiagnosticCode::ALL`, so dropping a class
/// here, or registering one without a row, fails the test.
const CLASSES: &[Class] = &[
    Class {
        code: "invalid_package_namespace",
        fixtures: &["package-invalid-namespace"],
    },
    Class {
        code: "invalid_wire_format",
        fixtures: &[
            "package-unknown-field",
            "package-wire-depth-over",
            "expression-invalid-wire",
        ],
    },
    Class {
        code: "invalid_schema_version",
        fixtures: &["package-invalid-schema"],
    },
    Class {
        code: "invalid_identifier",
        fixtures: &["package-invalid-identifier"],
    },
    Class {
        code: "invalid_requirement_revision",
        fixtures: &[
            "package-invalid-requirement-revision",
            "package-invalid-requirement-revision-over",
        ],
    },
    Class {
        code: "invalid_source_revision",
        fixtures: &[
            "package-invalid-source-revision",
            "package-invalid-source-revision-over",
        ],
    },
    Class {
        code: "duplicate_requirement",
        fixtures: &["package-duplicate-requirement"],
    },
    Class {
        code: "duplicate_clause",
        fixtures: &["package-duplicate-clause"],
    },
    Class {
        code: "cross_package_reference",
        fixtures: &["package-cross-reference", "coverage-cross-package"],
    },
    Class {
        code: "invalid_source_span",
        fixtures: &["package-invalid-span", "package-invalid-span-offset-over"],
    },
    Class {
        code: "floating_executable_clause",
        fixtures: &["package-floating"],
    },
    Class {
        code: "informational_clause_anchored",
        fixtures: &["package-information-anchored"],
    },
    Class {
        code: "incompatible_clause_anchor",
        fixtures: &["package-incompatible-anchor"],
    },
    Class {
        code: "malformed_reference",
        fixtures: &["package-malformed-reference"],
    },
    Class {
        code: "stale_requirement_revision",
        fixtures: &["package-stale-reference", "coverage-stale"],
    },
    Class {
        code: "orphaned_requirement_reference",
        fixtures: &["package-orphan-reference", "coverage-missing"],
    },
    Class {
        code: "orphaned_clause_reference",
        fixtures: &["package-orphan-clause"],
    },
    Class {
        code: "duplicate_type_declaration",
        fixtures: &["expression-duplicate-type"],
    },
    Class {
        code: "duplicate_value_declaration",
        fixtures: &["expression-duplicate-value"],
    },
    Class {
        code: "duplicate_function_declaration",
        fixtures: &["expression-duplicate-function"],
    },
    Class {
        code: "duplicate_field",
        fixtures: &["expression-duplicate-field"],
    },
    Class {
        code: "duplicate_variant",
        fixtures: &["expression-duplicate-variant"],
    },
    Class {
        code: "duplicate_parameter",
        fixtures: &["expression-duplicate-parameter"],
    },
    Class {
        code: "empty_enum",
        fixtures: &["expression-empty-enum"],
    },
    Class {
        code: "invalid_numeric_bounds",
        fixtures: &[
            "expression-invalid-numeric",
            "expression-rational-zero-denominator",
        ],
    },
    Class {
        code: "text_bound_exceeded",
        fixtures: &["expression-text-over"],
    },
    Class {
        code: "unbounded_collection",
        fixtures: &["expression-unbounded-collection"],
    },
    Class {
        code: "collection_bound_exceeded",
        fixtures: &["expression-collection-bound"],
    },
    Class {
        code: "orphaned_type_reference",
        fixtures: &["expression-orphan-type"],
    },
    Class {
        code: "recursive_type",
        fixtures: &["expression-recursive-type"],
    },
    Class {
        code: "orphaned_value_reference",
        fixtures: &["expression-orphan-value"],
    },
    Class {
        code: "orphaned_function_reference",
        fixtures: &["expression-orphan-function"],
    },
    Class {
        code: "invalid_state_observation",
        fixtures: &["expression-invalid-state"],
    },
    Class {
        code: "invalid_scope",
        fixtures: &["expression-invalid-scope"],
    },
    Class {
        code: "arity_mismatch",
        fixtures: &["expression-arity"],
    },
    Class {
        code: "ill_typed_expression",
        fixtures: &["expression-ill-typed"],
    },
    Class {
        code: "result_type_mismatch",
        fixtures: &["expression-result-type"],
    },
    Class {
        code: "non_boolean_clause_root",
        fixtures: &["expression-non-boolean-root"],
    },
    Class {
        code: "potentially_undefined",
        fixtures: &[
            "expression-obligation-option-presence",
            "expression-obligation-non-zero-divisor",
            "expression-obligation-index-in-bounds",
            "expression-obligation-checked-range",
        ],
    },
    Class {
        code: "expression_too_large",
        fixtures: &["expression-nodes-over"],
    },
    Class {
        code: "unsupported_schema_version",
        fixtures: &[
            "package-schema-1-0",
            "package-schema-1-2",
            "package-schema-2-0",
        ],
    },
    Class {
        code: "canonicalization_resource_exhausted",
        fixtures: &["package-canonical-limit"],
    },
    Class {
        code: "duplicate_artifact_trace",
        fixtures: &["coverage-duplicate"],
    },
    Class {
        code: "semantic_input_too_large",
        fixtures: &[
            "expression-semantic-nodes-over",
            "expression-depth-over",
            "expression-type-depth-over",
        ],
    },
];

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/contract-v0.1")
}

fn schemas() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

/// Runs the whole corpus. A fixture whose execution panics is reported by the
/// runner as an error, so `unwrap` fails the calling test for it. Whether each
/// fixture still equals its frozen expectation is TC-018's gate, not read here:
/// the NFR-003 criteria are asserted on each row's `actual` below.
fn run() -> Vec<Value> {
    let results: Vec<FixtureResult> =
        run_corpus(&corpus(), &schemas()).expect("no fixture of the corpus panics or fails to run");
    results
        .iter()
        .map(|result| serde_json::to_value(result).unwrap())
        .collect()
}

fn row<'a>(rows: &'a [Value], fixture: &str) -> &'a Value {
    rows.iter()
        .find(|row| row["fixture_id"] == fixture)
        .unwrap_or_else(|| panic!("corpus has no fixture {fixture}"))
}

fn diagnostic_codes(row: &Value) -> Vec<&str> {
    row["actual"]["diagnostics"]
        .as_array()
        .unwrap_or_else(|| panic!("fixture {} has no diagnostics list", row["fixture_id"]))
        .iter()
        .map(|diagnostic| diagnostic["code"].as_str().unwrap())
        .collect()
}

/// The codes STD-001's four semantic code tables list, read from the spec text.
fn std001_declared_codes() -> BTreeSet<String> {
    let text = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("spec/core/functional/STD-001-diagnostic-registry.md"),
    )
    .unwrap();
    let start = text.find("## Identity and Reference Codes").unwrap();
    let end = text.find("## Typed Code Form").unwrap();
    text[start..end]
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|rest| rest.split_once('`'))
        .map(|(code, _)| code.to_owned())
        .collect()
}

/// The package JSON text a package fixture hands the decoder.
fn package_text(input: &Value) -> String {
    match input.get("document_json") {
        Some(Value::String(text)) => text.clone(),
        _ => serde_json::to_string(input).unwrap(),
    }
}

fn input(fixture: &str) -> Value {
    serde_json::from_slice(
        &fs::read(corpus().join("inputs").join(format!("{fixture}.json"))).unwrap(),
    )
    .unwrap()
}

/// The class table is the STD-001 registry: same codes, no duplicates, and
/// every fixture it names exists in the corpus.
///
/// Trace: TC-019, NFR-003-AC-1
#[trace("TC-019", "NFR-003-AC-1")]
#[test]
fn declared_classes_match_std001() {
    let table: Vec<&str> = CLASSES.iter().map(|class| class.code).collect();
    let table_set: BTreeSet<&str> = table.iter().copied().collect();
    assert_eq!(table.len(), table_set.len(), "a class appears twice");
    let registry: BTreeSet<&str> = DiagnosticCode::ALL.iter().map(|c| c.as_str()).collect();
    assert_eq!(table_set, registry);
    assert_eq!(table.len(), 44);
    let spec = std001_declared_codes();
    assert_eq!(
        spec.iter().map(String::as_str).collect::<BTreeSet<_>>(),
        registry,
        "STD-001's semantic code tables and DiagnosticCode::ALL must name the same classes"
    );
    for class in CLASSES {
        assert!(!class.fixtures.is_empty(), "{} has no fixture", class.code);
        for fixture in class.fixtures {
            assert!(
                corpus()
                    .join("inputs")
                    .join(format!("{fixture}.json"))
                    .is_file(),
                "{fixture} is not a corpus input"
            );
        }
    }
}

/// Trace: TC-019, NFR-003-AC-1
#[trace("TC-019", "NFR-003-AC-1")]
#[test]
fn tc_019_every_declared_invalid_input_class_returns_a_stable_code_and_no_panic() {
    let rows = run();
    let mut reached = BTreeSet::new();
    for class in CLASSES {
        // The code is a registered, typed `DiagnosticCode` spelled as declared.
        let typed: DiagnosticCode =
            serde_json::from_value(Value::String(class.code.to_owned())).unwrap();
        assert_eq!(typed.as_str(), class.code);
        for fixture in class.fixtures {
            let row = row(&rows, fixture);
            let codes = diagnostic_codes(row);
            assert!(
                codes.contains(&class.code),
                "{fixture}: expected {} among {codes:?}",
                class.code
            );
            for diagnostic in row["actual"]["diagnostics"].as_array().unwrap() {
                assert_eq!(diagnostic["severity"], "error");
                let code = diagnostic["code"].as_str().unwrap();
                assert!(
                    DiagnosticCode::ALL.iter().any(|c| c.as_str() == code),
                    "{fixture}: unregistered code {code}"
                );
            }
            reached.insert(class.code);
        }
    }
    assert_eq!(reached.len(), DiagnosticCode::ALL.len());

    // Decoding straight through the public API: a wire-level class returns its
    // code and does not panic.
    for (fixture, code) in [
        ("package-unknown-field", DiagnosticCode::InvalidWireFormat),
        ("package-wire-depth-over", DiagnosticCode::InvalidWireFormat),
        (
            "package-invalid-namespace",
            DiagnosticCode::InvalidPackageNamespace,
        ),
        (
            "package-invalid-schema",
            DiagnosticCode::InvalidSchemaVersion,
        ),
        (
            "package-schema-2-0",
            DiagnosticCode::UnsupportedSchemaVersion,
        ),
        (
            "package-invalid-identifier",
            DiagnosticCode::InvalidIdentifier,
        ),
        (
            "package-invalid-requirement-revision-over",
            DiagnosticCode::InvalidRequirementRevision,
        ),
        ("package-invalid-span", DiagnosticCode::InvalidSourceSpan),
    ] {
        let errors = ContractPackage::from_json_str(
            &package_text(&input(fixture)),
            ValidationOptions::strict(),
        )
        .expect_err(fixture);
        assert_eq!(errors[0].code, code, "{fixture}");
    }

    // Hostile bytes that are not a package at all.
    let deep = "[".repeat(100_000);
    let hostile: [&[u8]; 6] = [
        b"",
        &[0xff, 0xfe],
        b"null",
        b"[]",
        b"{\"id\":",
        deep.as_bytes(),
    ];
    for bytes in hostile {
        let errors = ContractPackage::from_json_bytes(bytes, ValidationOptions::strict())
            .expect_err("hostile bytes are not a package");
        assert_eq!(errors[0].code, DiagnosticCode::InvalidWireFormat);
    }
}

/// Trace: TC-019, NFR-003-AC-2
#[trace("TC-019", "NFR-003-AC-2")]
#[test]
fn tc_019_no_invalid_input_contributes_canonical_identity_or_covered_status() {
    let rows = run();

    // Controls: the same runner does produce canonical identity and covered
    // status for admitted input, so the absence below is not vacuous.
    let admitted = row(&rows, "package-reference");
    assert_eq!(admitted["actual"]["valid"], true);
    assert!(!admitted["actual"]["canonical"]
        .as_array()
        .unwrap()
        .is_empty());
    let deep = row(&rows, "coverage-deep");
    assert!(deep["actual"]["coverage"]["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .any(|requirement| requirement["class"] == "deep"));

    for class in CLASSES {
        for fixture in class.fixtures {
            let row = row(&rows, fixture);
            let actual = &row["actual"];
            assert!(!diagnostic_codes(row).is_empty(), "{fixture}");
            match row["operation"].as_str().unwrap() {
                "package" | "expression" => {
                    assert_eq!(actual["valid"], false, "{fixture}");
                    assert_eq!(actual["canonical"], Value::Array(vec![]), "{fixture}");
                    assert_eq!(actual["dependencies"], Value::Array(vec![]), "{fixture}");
                }
                "coverage" => {
                    // The invalid trace is an orphaned artifact row and no
                    // requirement row is shallow or deep because of it.
                    let artifacts = actual["coverage"]["artifacts"].as_array().unwrap();
                    assert!(!artifacts.is_empty(), "{fixture}");
                    for artifact in artifacts {
                        assert_eq!(artifact["class"], "orphaned", "{fixture}");
                    }
                    for requirement in actual["coverage"]["requirements"].as_array().unwrap() {
                        assert_eq!(requirement["class"], "uncovered", "{fixture}");
                    }
                }
                other => panic!("{fixture}: unexpected operation {other}"),
            }
        }
    }

    // Decoding rejects the input, so no package exists to canonicalize or to
    // take an identity from.
    for class in CLASSES {
        for fixture in class.fixtures.iter().filter(|f| {
            matches!(
                f.strip_prefix("package-"),
                Some(
                    "unknown-field"
                        | "wire-depth-over"
                        | "invalid-namespace"
                        | "invalid-schema"
                        | "schema-1-0"
                        | "schema-1-2"
                        | "schema-2-0"
                        | "invalid-identifier"
                        | "invalid-requirement-revision"
                        | "invalid-requirement-revision-over"
                        | "invalid-source-revision"
                        | "invalid-source-revision-over"
                        | "invalid-span"
                        | "invalid-span-offset-over"
                )
            )
        }) {
            let decoded = ContractPackage::from_json_str(
                &package_text(&input(fixture)),
                ValidationOptions::strict(),
            );
            assert!(decoded.is_err(), "{fixture} decoded into a package");
        }
    }
}
