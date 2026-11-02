// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-107, AC-112, AC-113, AC-176 and AC-310: QSpec's CheckedPackage V2 fixtures and
//! vectors against the production reader.
//!
//! AC-107: the positive fixtures admit end to end. AC-112: every mutation of
//! `adverse.json` refuses as recorded, with no identity refreshed, bar the
//! listed expected failures, and each body-grammar `flattened` form is not
//! refused `malformed_wire`. AC-113: dependency selections name exact
//! structural package supplies. AC-310: retired package identity members
//! refuse at their exact pointers, while the single-body positives retain
//! their structural names and node ids.
//!
//! The fixtures are QSpec's and are read from a `quire-specification` checkout
//! named by `QUIRE_SPECIFICATION_DIR`; nothing of them is copied into this
//! repository. This target is outside the run of `make test` (its manifest
//! entry has `test = false`, and a plain `make test` has no checkout). CI
//! compiles this target without running it. `make conformance-qspec` runs its
//! fixture tests; they fail, and never skip, when the
//! variable is unset or empty, when it names a path that does not hold
//! `proposals/checked-package-v2/fixtures/`, when a file cannot be read or is
//! not JSON, when a list is absent or empty, and when a result is not the
//! recorded one.

use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedArtifactRef, CheckedExtensionClauseSchema,
    CheckedExtensionDefinition, CheckedExtensionGrammarSchema, CheckedExtensionNodeSchema,
    CheckedExtensionProduction, CheckedExtensionTypedNodeSchema, CheckedLiteralValue,
    CheckedNodeId, CheckedNodeTag, CheckedOperationMember, CheckedPackageEvidence,
    CheckedPackageEvidenceInsertionError, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm, CompleteLoweringProfileV2, ContractDependencies, ContractPackageReadResult,
    ContractPackageWireV1,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod dependency_call;
mod dimension_equivalence;
mod extension;
mod fairness_targets;
#[path = "../it/support/checked_package.rs"]
mod fixture_support;
mod grouped_ordinal;
mod hybrid;
mod model_option_types;
mod model_root;
mod model_tuple_union;
mod node_index;
mod node_ordinal;
mod probabilistic_claims;
mod projection_read_form;
mod protocol_locations;
mod protocol_vectors;
mod reaches_population;
mod selection_vectors;
mod single_body;
mod temporal_clock_bindings;
mod timed_model;
mod timed_slots;
mod transition;

/// The environment variable naming the root of a `quire-specification` checkout.
const SPECIFICATION_DIR: &str = "QUIRE_SPECIFICATION_DIR";

/// Where QSpec publishes the CheckedPackage V2 fixtures, from its root.
const FIXTURES: &str = "proposals/checked-package-v2/fixtures";

/// The complete published positive set required by FR-038-AC-107 and AC-176.
const POSITIVE_ALL: [&str; 19] = [
    "positive-all-families.json",
    "positive-clause-operations.json",
    "positive-control-operations.json",
    "positive-dependency-call.json",
    "positive-dependency-library.json",
    "positive-extension-nodes.json",
    "positive-integer-literal-references.json",
    "positive-model-tuple-union.json",
    "positive-nominal-identities.json",
    "positive-operation-identities.json",
    "positive-recursive-records.json",
    "positive-state-transition.json",
    "positive-tick-timed-model-nodes.json",
    "positive-timed-clause-nodes.json",
    "positive-timed-model-nodes.json",
    "positive-timed-slot-nodes.json",
    "positive-two-owners-a.json",
    "positive-two-owners-b.json",
    "positive-union-nodes.json",
];

/// The selected domain document in the proposal.
const DOMAIN_DOCUMENT: &str = "domain-package-acme-orders.json";

/// Read QSpec's authoritative fixture-to-document map; a declared selection
/// cannot silently fall back to the baseline document.
fn vector_selected_documents(proposal: &Path) -> Result<BTreeMap<String, String>, String> {
    let mapping = read_json(&proposal.join("selected-documents.json"))?;
    selected_document_assignments(&mapping, proposal)
}

fn selected_document_assignments(
    mapping: &Value,
    proposal: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let assignments = mapping["selected_documents"]
        .as_object()
        .ok_or("selected-documents.json: selected_documents is not an object")?;
    if assignments.is_empty() {
        return Err("selected-documents.json: selected_documents is empty".into());
    }
    let mut documents = BTreeMap::new();
    for (fixture, document) in assignments {
        let name = fixture
            .strip_prefix("fixtures/")
            .ok_or_else(|| format!("selected-documents.json: invalid fixture path {fixture:?}"))?;
        if !POSITIVE_ALL.contains(&name) {
            return Err(format!(
                "selected-documents.json: unlisted positive fixture {fixture}"
            ));
        }
        let document = document.as_str().ok_or_else(|| {
            format!("selected-documents.json: {fixture} document is not a string")
        })?;
        if !document.starts_with("domain-package-")
            || !document.ends_with(".json")
            || document.contains('/')
            || document.contains('\\')
        {
            return Err(format!(
                "selected-documents.json: invalid document {document:?}"
            ));
        }
        read_json(&proposal.join(document))?;
        documents.insert(name.to_owned(), document.to_owned());
    }
    Ok(documents)
}

/// Ordinary positives use QSpec's baseline Orders document. Missing or invalid
/// document selections fail in `vector_selected_documents` before this lookup.
fn positive_document<'a>(name: &str, documents: &'a BTreeMap<String, String>) -> &'a str {
    documents
        .get(name)
        .map(String::as_str)
        .unwrap_or(DOMAIN_DOCUMENT)
}

#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn tc_048_positive_document_selection_follows_qspec_map() {
    let proposal = proposal_dir_of(&fixtures_dir());
    vector_selected_documents(&proposal).expect("published document selections");
    let fixture = "fixtures/positive-model-tuple-union.json";
    let mut invalid = json!({"selected_documents": {fixture: DOMAIN_DOCUMENT}});
    let documents = selected_document_assignments(&invalid, &proposal).expect("synthetic map");
    assert_eq!(
        positive_document("positive-model-tuple-union.json", &documents),
        DOMAIN_DOCUMENT
    );
    assert_eq!(
        positive_document("positive-recursive-records.json", &documents),
        DOMAIN_DOCUMENT
    );
    invalid["selected_documents"][fixture] = json!("../other.json");
    assert!(selected_document_assignments(&invalid, &proposal).is_err());
    invalid["selected_documents"][fixture] = json!(DOMAIN_DOCUMENT);
    assert!(selected_document_assignments(&invalid, &proposal).is_ok());
    invalid
        .as_object_mut()
        .expect("mapping")
        .remove("selected_documents");
    assert!(selected_document_assignments(&invalid, &proposal).is_err());
}

/// QSpec's recursive records, including the owner-free derived nodes in each group.
const RECURSIVE: &str = "positive-recursive-records.json";

/// The adverse mutations FR-038-AC-112 applies, under `fixtures/`.
const ADVERSE: &str = "adverse.json";

/// The package FR-038-AC-112 mutates, under `fixtures/`.
const MUTATED_BASE: &str = "positive-all-families.json";

/// The fixtures directory of the checkout `raw` names, or why it names none:
/// `raw` unset or empty, or a path holding no fixtures directory.
fn fixtures_dir_of(raw: Option<&str>) -> Result<PathBuf, String> {
    let root = raw.unwrap_or_default();
    if root.is_empty() {
        return Err(format!(
            "{SPECIFICATION_DIR} is unset or empty: set it to the root of a quire-specification checkout"
        ));
    }
    let dir = PathBuf::from(root).join(FIXTURES);
    if !dir.is_dir() {
        return Err(format!(
            "{SPECIFICATION_DIR}={root} holds no {FIXTURES}/: it is not a quire-specification checkout"
        ));
    }
    Ok(dir)
}

/// The fixtures directory of the named checkout; a test failure naming the
/// variable when it is unset, empty or names no fixtures directory.
fn fixtures_dir() -> PathBuf {
    fixtures_dir_of(std::env::var(SPECIFICATION_DIR).ok().as_deref())
        .unwrap_or_else(|why| panic!("{why}"))
}

/// The proposal directory, the parent of the fixtures directory.
fn proposal_dir_of(fixtures: &Path) -> PathBuf {
    fixtures.join("..")
}

/// The JSON at `path`, or why it cannot be read.
fn read_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read(path)
        .map_err(|error| format!("{} cannot be read: {error}", path.display()))?;
    serde_json::from_slice(&text)
        .map_err(|error| format!("{} is not JSON: {error}", path.display()))
}

/// Read `package` through the production reader with its declared features.
fn read_package(package: &Value) -> Result<CheckedPackageV2ReadResult, String> {
    let document = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    read_package_with_document(package, "checked package", &document)
}

/// The one evidence-backed production read used by old and new conformance cases.
fn read_package_with_document(
    package: &Value,
    name: &str,
    domain_document: &Path,
) -> Result<CheckedPackageV2ReadResult, String> {
    read_package_with_document_and_extensions(package, name, domain_document, true)
}

fn retention_definition() -> CheckedExtensionDefinition {
    CheckedExtensionDefinition {
        grammar_schema: CheckedExtensionGrammarSchema {
            productions: vec![CheckedExtensionProduction {
                keyword: "retention".into(),
                clauses: vec!["subject".into(), "days".into()],
            }],
        },
        typed_node_schema: CheckedExtensionTypedNodeSchema {
            nodes: vec![CheckedExtensionNodeSchema {
                keyword: "retention".into(),
                node_kind: "retention".into(),
                clauses: vec![
                    CheckedExtensionClauseSchema {
                        name: "subject".into(),
                        type_ref: "Text".into(),
                    },
                    CheckedExtensionClauseSchema {
                        name: "days".into(),
                        type_ref: "Int[1, 3650]".into(),
                    },
                ],
            }],
        },
    }
}

fn quota_definition() -> CheckedExtensionDefinition {
    CheckedExtensionDefinition {
        grammar_schema: CheckedExtensionGrammarSchema {
            productions: vec![CheckedExtensionProduction {
                keyword: "quota".into(),
                clauses: vec!["limit".into()],
            }],
        },
        typed_node_schema: CheckedExtensionTypedNodeSchema {
            nodes: vec![CheckedExtensionNodeSchema {
                keyword: "quota".into(),
                node_kind: "quota".into(),
                clauses: vec![CheckedExtensionClauseSchema {
                    name: "limit".into(),
                    type_ref: "Int[1, 100]".into(),
                }],
            }],
        },
    }
}

fn policy_definition() -> CheckedExtensionDefinition {
    CheckedExtensionDefinition {
        grammar_schema: CheckedExtensionGrammarSchema {
            productions: vec![CheckedExtensionProduction {
                keyword: "policy".into(),
                clauses: vec!["retain".into()],
            }],
        },
        typed_node_schema: CheckedExtensionTypedNodeSchema {
            nodes: vec![CheckedExtensionNodeSchema {
                keyword: "policy".into(),
                node_kind: "policy".into(),
                clauses: vec![CheckedExtensionClauseSchema {
                    name: "retain".into(),
                    type_ref: "retention".into(),
                }],
            }],
        },
    }
}

fn read_package_with_document_and_extensions(
    package: &Value,
    name: &str,
    domain_document: &Path,
    hold_retention: bool,
) -> Result<CheckedPackageV2ReadResult, String> {
    read_package_with_document_and_extensions_at_limit(
        package,
        name,
        domain_document,
        hold_retention,
        CheckedPackageReadLimits::bounded(),
    )
}

fn read_package_with_document_and_extensions_at_limit(
    package: &Value,
    name: &str,
    domain_document: &Path,
    hold_retention: bool,
    limits: CheckedPackageReadLimits,
) -> Result<CheckedPackageV2ReadResult, String> {
    read_package_with_document_override(
        package,
        name,
        domain_document,
        hold_retention,
        limits,
        None,
    )
}

fn read_package_with_document_override(
    package: &Value,
    name: &str,
    domain_document: &Path,
    hold_retention: bool,
    limits: CheckedPackageReadLimits,
    document_override: Option<&[u8]>,
) -> Result<CheckedPackageV2ReadResult, String> {
    // The reader takes canonical bytes; the published files are indented.
    let bytes = serde_json::to_vec(package).map_err(|error| error.to_string())?;
    let mut evidence = CheckedPackageEvidence::new();
    if hold_retention
        && package["lock"]["definition_selections"]
            .as_array()
            .is_some_and(|rows| {
                rows.iter()
                    .any(|row| row["authority"] == "example" && row["identity"] == "retention")
            })
    {
        evidence
            .insert_extension_definition(
                CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "retention".into(),
                },
                retention_definition(),
            )
            .map_err(|error| error.to_string())?;
    }
    if package["lock"]["definition_selections"]
        .as_array()
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row["authority"] == "example" && row["identity"] == "quota")
        })
    {
        evidence
            .insert_extension_definition(
                CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "quota".into(),
                },
                quota_definition(),
            )
            .map_err(|error| error.to_string())?;
    }
    if package["lock"]["definition_selections"]
        .as_array()
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row["authority"] == "example" && row["identity"] == "policy")
        })
    {
        evidence
            .insert_extension_definition(
                CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "policy".into(),
                },
                policy_definition(),
            )
            .map_err(|error| error.to_string())?;
    }
    for feature in package["lock"]["required_features"]
        .as_array()
        .ok_or("lock.required_features is not an array")?
    {
        evidence.support_feature(
            feature
                .as_str()
                .ok_or("a required feature is not a string")?,
        );
    }
    if package["lock"]["dependency_selections"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["identity"] == "test/predicates"))
    {
        let dependency = read_json(&fixtures_dir().join("positive-dependency-library.json"))?;
        let CheckedPackageV2ReadResult::Admitted(dependency) =
            read_package_with_document(&dependency, "published dependency", domain_document)?
        else {
            return Err("published dependency library did not admit".into());
        };
        evidence
            .insert_dependency_package("test/predicates", dependency)
            .map_err(|error| format!("dependency evidence insertion failed: {error}"))?;
    }
    // Supply the one domain document this harness owns. Leave malformed or
    // unselected root owners to the reader's own refusal path.
    let roots = package["semantic_graph"]["nodes"]
        .as_array()
        .ok_or_else(|| format!("{name}: semantic_graph.nodes is not an array"))?
        .iter()
        .filter(|node| node["semantic_form"] == "model_root");
    for root in roots {
        let Some(identity) = root["owner"]["identity"].as_str() else {
            continue;
        };
        if identity != "acme/orders" && identity != "acme/shapes" {
            continue;
        }
        let selected_document = if identity == "acme/shapes" {
            proposal_dir_of(&fixtures_dir()).join("domain-package-acme-shapes.json")
        } else {
            domain_document.to_path_buf()
        };
        let document = if identity == "acme/orders" {
            match document_override {
                Some(bytes) => bytes.to_vec(),
                None => std::fs::read(&selected_document).map_err(|error| {
                    format!("{} cannot be read: {error}", selected_document.display())
                })?,
            }
        } else {
            std::fs::read(&selected_document).map_err(|error| {
                format!("{} cannot be read: {error}", selected_document.display())
            })?
        };
        serde_json::from_slice::<Value>(&document)
            .map_err(|error| format!("{} is not JSON: {error}", selected_document.display()))?;
        evidence
            .insert_domain_package_document(identity, document)
            .map_err(|error| format!("{name}: evidence insertion failed: {error}"))?;
    }
    Ok(CheckedPackageV2::read(&bytes, limits, &evidence))
}

/// What the production reader makes of `package`: `admitted`, `incomplete`, or
/// `refused:<code>` with `/<cause>` where the reader gives one.
fn reading_of(package: &Value) -> Result<String, String> {
    Ok(match read_package(package)? {
        CheckedPackageV2ReadResult::Admitted(_) => "admitted".to_owned(),
        CheckedPackageV2ReadResult::Incomplete(_) => "incomplete".to_owned(),
        CheckedPackageV2ReadResult::Refused(refusal) => match refusal.cause {
            Some(cause) => format!("refused:{}/{}", code_word(refusal.code), cause_word(cause)),
            None => format!("refused:{}", code_word(refusal.code)),
        },
    })
}

/// The wire word of a refusal code. No wildcard: a variant added to the reader
/// is a compile error here, so no refusal can read as a stale word.
fn code_word(code: Code) -> &'static str {
    match code {
        Code::UnknownContractVersion => "unknown_contract_version",
        Code::MalformedWire => "malformed_wire",
        Code::DuplicateMember => "duplicate_member",
        Code::UnknownMember => "unknown_member",
        Code::NoncanonicalWire => "noncanonical_wire",
        Code::UnknownRequiredCapability => "unknown_required_capability",
        Code::InvalidSemanticGraph => "invalid_semantic_graph",
        Code::InvalidSourceMap => "invalid_source_map",
        Code::UnsupportedNodeTag => "unsupported_node_tag",
        Code::MissingDeclaration => "missing_declaration",
        Code::InvalidModelBinding => "invalid_model_binding",
        Code::InvalidPackage => "invalid_package",
        Code::IllTyped => "ill_typed",
        Code::WrongSnapshot => "wrong_snapshot",
        Code::AmbiguousDeclaration => "ambiguous_declaration",
        Code::MissingImport => "missing_import",
        Code::UnknownProfile => "unknown_profile",
        Code::UnsupportedConstruct => "unsupported_construct",
    }
}

/// The wire word of a refusal cause, with no wildcard for the reason above.
fn cause_word(cause: Cause) -> &'static str {
    match cause {
        Cause::MalformedJson => "malformed-json",
        Cause::MissingMember => "missing-member",
        Cause::UnknownMember => "unknown-member",
        Cause::WrongValueKind => "wrong-value-kind",
        Cause::DefinitionCycle => "definition-cycle",
        Cause::IncompatibleDefinition => "incompatible-definition",
        Cause::FeatureSetMismatch => "feature-set-mismatch",
        Cause::InsufficientNextCharge => "insufficient-next-charge",
        Cause::MissingName => "missing-name",
        Cause::MalformedDeclaration => "malformed-declaration",
        Cause::UnknownOperation => "unknown-operation",
        Cause::OperationClassMismatch => "operation-class-mismatch",
        Cause::OperationLawMissing => "operation-law-missing",
        Cause::OperationLawMismatch => "operation-law-mismatch",
        Cause::OperationLawUnselected => "operation-law-unselected",
        Cause::OperationModeMismatch => "operation-mode-mismatch",
        Cause::OperationModeTypeMismatch => "operation-mode-type-mismatch",
        Cause::OperationMemberMismatch => "operation-member-mismatch",
        Cause::OperatorIneligible => "operator-ineligible",
        Cause::DeclarationNominalMismatch => "declaration-nominal-mismatch",
        Cause::AmbiguousName => "ambiguous-name",
        Cause::MissingSelection => "missing-selection",
        Cause::WrongModelSelection => "wrong-model-selection",
        Cause::ConflictingBinding => "conflicting-binding",
        Cause::UnpreservedModelMeaning => "unpreserved-model-meaning",
        Cause::ConflictingDefinition => "conflicting-definition",
        Cause::InvalidSemanticGraph => "invalid-semantic-graph",
        Cause::InvalidValue => "invalid-value",
        Cause::InvalidDerivedShape => "invalid-derived-shape",
        Cause::DuplicateMember => "duplicate-member",
        Cause::TypeMismatch => "type-mismatch",
        Cause::NonBooleanRoot => "non-boolean-root",
        Cause::MissingTimedModel => "missing-timed-model",
        Cause::TimedNodeReference => "timed-node-reference",
        Cause::DuplicateClockPlace => "duplicate-clock-place",
        Cause::MalformedClockPlace => "malformed-clock-place",
        Cause::InvalidTimeQuantity => "invalid-time-quantity",
        Cause::ClockWithoutDenseTime => "clock-without-dense-time",
        Cause::UnsupportedSelection => "unsupported-selection",
        Cause::WrongSelectionRole => "wrong-selection-role",
        Cause::InexactInteger => "inexact-integer",
        Cause::InexactNumber => "inexact-number",
        Cause::ExpressionForm => "expression-form",
        Cause::DeclarationForm => "declaration-form",
        Cause::WrongExport => "wrong-export",
        Cause::WrongObservation => "wrong-observation",
    }
}

/// Whether the reader's `given` reading is the refusal QSpec's `recorded`
/// outcome names: the same code, and the same cause only where the outcome
/// gives one (`refused:<code>` or `refused:<code>/<cause>`, FR-038-AC-112).
fn refuses_as(given: &str, recorded: &str) -> bool {
    if recorded.contains('/') {
        given == recorded
    } else {
        given.split('/').next() == Some(recorded)
    }
}

/// A mutation the reader does not yet refuse as QSpec records it.
struct ExpectedFailure {
    /// The mutation `id` in `adverse.json`.
    id: &'static str,
    /// What the reader gives it today, which differs from the recorded
    /// `outcome`.
    refusal: &'static str,
    /// The open ticket that owns the missing refusal. Delete the entry when it
    /// lands: a listed id the reader refuses as recorded fails the run.
    ticket: &'static str,
}

/// FR-038-AC-112's expected-failure list: the mutations the reader does not yet
/// refuse as recorded, each with the refusal it gives and the open ticket that
/// owns the missing one. It is empty today: the reader enforces the flat v2
/// wire, so each of the five body-grammar mutations refuses `malformed_wire` as
/// recorded and none may be listed (FR-038-AC-118, checked by the test of the
/// result). A new exception for another mutation is an entry here; the stale-entry
/// rule of AC-112 holds for it.
const EXPECTED_FAILURES: &[ExpectedFailure] = &[];

/// The replacement of `package` at `pointer`, or why it cannot be made.
fn replace_at(package: &mut Value, pointer: &str, replacement: &Value) -> Result<(), String> {
    let slot = package
        .pointer_mut(pointer)
        .ok_or_else(|| format!("pointer {pointer} resolves to nothing in the package"))?;
    *slot = replacement.clone();
    Ok(())
}

/// Sort fixture-authored structural ids by their typed wire order.
fn typed_node_id(value: &Value) -> CheckedNodeId {
    serde_json::from_value(value.clone()).expect("structural node id")
}

fn sort_node_ids(ids: &mut [Value]) {
    ids.sort_by_key(typed_node_id);
}

/// The mutation list `name` of `adverse`, non-empty, or why it is not.
fn mutations<'a>(adverse: &'a Value, name: &str) -> Result<&'a [Value], String> {
    match adverse.get(name).and_then(Value::as_array) {
        Some(list) if !list.is_empty() => Ok(list),
        _ => Err(format!("{ADVERSE} has no non-empty {name} list")),
    }
}

/// A string member of a mutation entry, or why it has none.
fn text_of<'a>(entry: &'a Value, member: &str) -> Result<&'a str, String> {
    entry
        .get(member)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("a mutation entry has no string {member}"))
}

/// FR-038-AC-112 over `base`, `adverse` and the expected-failure list: every
/// problem found, empty when the run passes. Applies each mutation to a fresh
/// copy of `base` with no identity refreshed.
fn adverse_problems(base: &Value, adverse: &Value, expected: &[ExpectedFailure]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut entries = Vec::new();
    for list in ["structural_mutations", "body_grammar_mutations"] {
        match mutations(adverse, list) {
            Ok(found) => entries.extend(found.iter().map(|entry| (list, entry))),
            Err(why) => problems.push(why),
        }
    }
    for (list, entry) in entries {
        let id = match text_of(entry, "id") {
            Ok(id) => id,
            Err(why) => {
                problems.push(why);
                continue;
            }
        };
        seen.insert(id);
        let step = (|| -> Result<(), String> {
            let pointer = text_of(entry, "pointer")?;
            let recorded = text_of(entry, "outcome")?;
            let replacement = entry.get("replacement").ok_or("no replacement")?;
            let mut mutated = base.clone();
            replace_at(&mut mutated, pointer, replacement)?;
            if list == "body_grammar_mutations" {
                let flattened = entry.get("flattened").ok_or("no flattened member")?;
                let mut control = base.clone();
                replace_at(&mut control, pointer, flattened)?;
                if reading_of(&control)?.starts_with("refused:malformed_wire") {
                    return Err("the flattened positive control is refused malformed_wire".into());
                }
            }
            let given = reading_of(&mutated)?;
            match expected.iter().find(|listed| listed.id == id) {
                None if !refuses_as(&given, recorded) => {
                    return Err(format!("read as {given}, not the recorded {recorded}"));
                }
                Some(listed) if refuses_as(&given, recorded) => {
                    return Err(format!(
                        "listed for {} but refused as recorded, {recorded}: delete the entry",
                        listed.ticket
                    ));
                }
                Some(listed) if given != listed.refusal => {
                    return Err(format!(
                        "read as {given}, neither the recorded {recorded} nor the listed {} ({})",
                        listed.refusal, listed.ticket
                    ));
                }
                _ => {}
            }
            Ok(())
        })();
        if let Err(why) = step {
            problems.push(format!("{id}: {why}"));
        }
    }
    for listed in expected {
        if !seen.contains(listed.id) {
            problems.push(format!("{}: listed but absent from {ADVERSE}", listed.id));
        }
    }
    problems
}

#[trace("TC-048", "FR-038-AC-107", "FR-038-AC-133")]
#[test]
fn tc_048_qspec_positive_fixtures_admit() {
    let dir = fixtures_dir();
    let proposal = proposal_dir_of(&dir);
    let documents = vector_selected_documents(&proposal).expect("published document selections");
    for name in POSITIVE_ALL {
        let document = proposal.join(positive_document(name, &documents));
        let (package, admitted) =
            read_positive(&dir, name, &document).unwrap_or_else(|why| panic!("{why}"));
        let _ = (&package, &admitted);
    }
}

#[trace("FR-349-AC-6", "FR-349-AC-8", "FR-349-AC-9")]
#[test]
fn qspec_positive_fixtures_expose_complete_public_typed_reads() {
    let dir = fixtures_dir();
    let proposal = proposal_dir_of(&dir);
    let documents = vector_selected_documents(&proposal).expect("published document selections");
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("authoritative operation catalog");
    let mut operations = 0;
    let mut constraints = 0;
    let mut leaves = 0;
    let mut literals = 0;
    let mut literal_kinds = std::collections::BTreeSet::new();
    for name in POSITIVE_ALL {
        let document = proposal.join(positive_document(name, &documents));
        let (wire, admitted) =
            read_positive(&dir, name, &document).unwrap_or_else(|why| panic!("{why}"));
        let wire_nodes = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("published nodes");
        assert_eq!(wire_nodes.len(), admitted.graph().nodes.len());
        for (wire_node, node) in wire_nodes.iter().zip(&admitted.graph().nodes) {
            assert_qspec_term_record(&node.body, &wire_node["body"], &catalog);
        }
        let mut pending: Vec<&CheckedTerm> = admitted
            .graph()
            .nodes
            .iter()
            .map(|node| &node.body)
            .collect();
        while let Some(term) = pending.pop() {
            match term {
                CheckedTerm::Literal {
                    type_id,
                    value_kind,
                    ..
                } => {
                    let read = term
                        .literal_read()
                        .expect("QSpec-admitted literal has a typed read");
                    assert_eq!(read.type_id, type_id);
                    assert_eq!(read.kind, *value_kind);
                    literal_kinds.insert(read.kind.as_wire());
                    literals += 1;
                }
                CheckedTerm::Application {
                    operation,
                    arguments,
                    ..
                } => {
                    let read = operation
                        .read()
                        .expect("QSpec-admitted operation has a complete typed read");
                    assert_eq!(Some(read.identity), operation.identity());
                    operations += 1;
                    constraints += read.constraints.len();
                    leaves += read.leaves.len();
                    pending.extend(arguments);
                }
                CheckedTerm::Aggregate { members } => pending.extend(members),
                CheckedTerm::Binding { value, .. } => pending.push(value),
                _ => {}
            }
        }
        let encoded = encode_checked_package_v2(
            admitted.name(),
            admitted.lock(),
            admitted.graph(),
            CheckedPackageSourceCorrespondenceRef {
                source_map: admitted.source_map(),
                refinement_step_rows: admitted.refinement_step_rows(),
            },
            quire_contract_model::CheckedPackageSidecarsRef {
                operation_effects: None,
                checked_term_data: admitted.checked_term_data(),
                temporal_bindings: admitted.temporal_bindings(),
            },
            admitted.capability_report(),
            admitted.diagnostics(),
            quire_canonical::Limits::new(1 << 24),
        )
        .expect("QSpec admitted package encodes");
        assert_eq!(
            encoded,
            quire_canonical::to_vec(&wire, quire_canonical::Limits::new(1 << 24))
                .expect("QSpec canonical bytes"),
            "{name}: exact published bytes"
        );
    }
    assert!(operations > 0, "published fixtures contain applications");
    assert!(
        constraints > 0,
        "published catalog has exercised constraints"
    );
    assert!(leaves > 0, "published fixtures contain structural leaves");
    assert!(literals > 0, "published fixtures contain literals");
    assert_eq!(
        literal_kinds,
        [
            "boolean",
            "integer",
            "rational",
            "quantity",
            "decimal",
            "float32_bits",
            "float64_bits",
            "text",
            "enum"
        ]
        .into_iter()
        .collect(),
        "published positives exercise every named QSpec literal kind"
    );
}

fn assert_qspec_term_record(term: &CheckedTerm, wire: &Value, catalog: &Value) {
    match term {
        CheckedTerm::Literal { .. } => assert_qspec_literal_read(term, wire),
        CheckedTerm::Application {
            operation,
            arguments,
            ..
        } => {
            assert_qspec_operation_record(
                operation.read().expect("admitted operation"),
                &wire["operation"],
                catalog,
            );
            let wire_arguments = wire["arguments"].as_array().expect("ordered arguments");
            assert_eq!(arguments.len(), wire_arguments.len());
            for (argument, expected) in arguments.iter().zip(wire_arguments) {
                assert_qspec_term_record(argument, expected, catalog);
            }
        }
        CheckedTerm::Aggregate { members } => {
            let wire_members = wire["members"].as_array().expect("ordered members");
            assert_eq!(members.len(), wire_members.len());
            for (member, expected) in members.iter().zip(wire_members) {
                assert_qspec_term_record(member, expected, catalog);
            }
        }
        CheckedTerm::Binding { value, .. } => {
            assert_qspec_term_record(value, &wire["value"], catalog);
        }
        _ => {}
    }
}

fn assert_qspec_literal_read(term: &CheckedTerm, wire: &Value) {
    let read = term.literal_read().expect("admitted literal read");
    assert_eq!(serde_json::to_value(read.type_id).unwrap(), wire["type"]);
    assert_eq!(read.kind.as_wire(), wire["value_kind"].as_str().unwrap());
    let value = &wire["value"];
    match read.value {
        CheckedLiteralValue::Boolean(actual) => assert_eq!(Some(actual), value.as_bool()),
        CheckedLiteralValue::Integer(actual) => {
            let expected = value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.as_number().expect("integer wire").to_string());
            assert_eq!(actual.to_string(), expected);
        }
        CheckedLiteralValue::Rational {
            numerator,
            denominator,
        } => {
            let wire = value.as_str().expect("rational wire");
            let (expected_numerator, expected_denominator) =
                wire.split_once('/').unwrap_or((wire, "1"));
            assert_eq!(numerator.to_string(), expected_numerator);
            assert_eq!(denominator.to_string(), expected_denominator);
        }
        CheckedLiteralValue::Decimal { coefficient, scale } => {
            let wire = value.as_str().expect("decimal wire");
            let (whole, fraction) = wire.split_once('.').expect("decimal scale");
            assert_eq!(scale, fraction.len());
            let negative = whole.starts_with('-');
            let digits = format!("{}{fraction}", whole.trim_start_matches('-'));
            let magnitude = digits.trim_start_matches('0');
            let expected = if magnitude.is_empty() {
                "0".to_owned()
            } else if negative {
                format!("-{magnitude}")
            } else {
                magnitude.to_owned()
            };
            assert_eq!(coefficient.to_string(), expected);
        }
        CheckedLiteralValue::Float32Bits(actual) => {
            assert_eq!(format!("{actual:08x}"), value.as_str().unwrap())
        }
        CheckedLiteralValue::Float64Bits(actual) => {
            assert_eq!(format!("{actual:016x}"), value.as_str().unwrap())
        }
        CheckedLiteralValue::Text(actual)
        | CheckedLiteralValue::String(actual)
        | CheckedLiteralValue::Uuid(actual)
        | CheckedLiteralValue::Enum(actual) => assert_eq!(actual, value.as_str().unwrap()),
        CheckedLiteralValue::Timestamp(actual) => {
            assert_eq!(actual.to_string(), value.as_str().unwrap())
        }
        CheckedLiteralValue::None => assert!(value.is_null()),
    }
}

fn assert_qspec_operation_record(
    read: quire_contract_model::CheckedOperationRead<'_>,
    wire: &Value,
    catalog: &Value,
) {
    assert_eq!(
        read.identity,
        wire["identity"].as_str().expect("wire identity")
    );
    assert_eq!(
        read.mode.map(|mode| (mode.kind.as_wire(), mode.value)),
        wire["mode"].as_object().map(|mode| (
            mode["kind"].as_str().expect("mode kind"),
            mode["value"].as_str().expect("mode value")
        ))
    );
    let laws = wire["laws"].as_array().expect("ordered laws");
    assert_eq!(read.laws.len(), laws.len());
    for (law, expected) in read.laws.iter().zip(laws) {
        assert_eq!(law.role.as_wire(), expected["role"].as_str().unwrap());
        assert_eq!(
            law.definition.authority.as_ref(),
            expected["definition"]["authority"].as_str().unwrap()
        );
        assert_eq!(
            law.definition.identity.as_ref(),
            expected["definition"]["identity"].as_str().unwrap()
        );
    }
    let leaves = wire["leaves"].as_array().expect("ordered leaves");
    assert_eq!(read.leaves.len(), leaves.len());
    for (leaf, expected) in read.leaves.iter().zip(leaves) {
        assert_eq!(
            leaf.path
                .iter()
                .map(|part| part.as_ref())
                .collect::<Vec<&str>>(),
            expected["path"]
                .as_array()
                .unwrap()
                .iter()
                .map(|part| part.as_str().unwrap())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            leaf.mode
                .as_ref()
                .map(|mode| (mode.kind.as_wire(), mode.value.as_ref())),
            expected["mode"].as_object().map(|mode| (
                mode["kind"].as_str().unwrap(),
                mode["value"].as_str().unwrap()
            ))
        );
        assert_eq!(leaf.laws.len(), expected["laws"].as_array().unwrap().len());
        for (law, expected_law) in leaf.laws.iter().zip(expected["laws"].as_array().unwrap()) {
            assert_eq!(law.role.as_wire(), expected_law["role"].as_str().unwrap());
            assert_eq!(
                law.definition.authority.as_ref(),
                expected_law["definition"]["authority"].as_str().unwrap()
            );
            assert_eq!(
                law.definition.identity.as_ref(),
                expected_law["definition"]["identity"].as_str().unwrap()
            );
        }
    }
    let entry = catalog["operations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["identity"] == read.identity)
        .expect("catalog identity");
    let constraints = entry["constraints"]
        .as_array()
        .expect("ordered constraints");
    assert_eq!(
        read.constraints.len(),
        constraints.len(),
        "{}",
        read.identity
    );
    for (constraint, expected) in read.constraints.iter().zip(constraints) {
        assert_eq!(
            constraint.kind.as_wire(),
            expected["kind"].as_str().unwrap()
        );
        assert_eq!(
            constraint.operands,
            expected["operands"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap())
                .collect::<Vec<_>>()
        );
        assert_eq!(constraint.family.as_deref(), expected["family"].as_str());
    }
    let member = &wire["member"];
    let node = |value: &CheckedNodeId, key: &str| {
        assert_eq!(serde_json::to_value(value).unwrap(), member[key]);
    };
    match read.member {
        None => assert!(member.is_null()),
        Some(CheckedOperationMember::Field {
            declaration,
            name,
            read_form,
        }) => {
            assert_eq!(member["kind"], "field");
            node(declaration, "declaration");
            assert_eq!(member["name"], name.as_ref());
            assert_eq!(
                read_form.map(|form| match form {
                    quire_contract_model::CheckedFieldReadForm::Bare => "bare",
                    quire_contract_model::CheckedFieldReadForm::ExplicitDeref => "explicit_deref",
                }),
                member["read_form"].as_str()
            );
        }
        Some(CheckedOperationMember::Position {
            declaration,
            position,
        }) => {
            assert_eq!(member["kind"], "position");
            node(declaration, "declaration");
            assert_eq!(
                position.to_string(),
                member["position"]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| member["position"].to_string())
            );
        }
        Some(CheckedOperationMember::Element { declaration }) => {
            assert_eq!(member["kind"], "element");
            node(declaration, "declaration");
        }
        Some(CheckedOperationMember::RelationshipEnd {
            declaration,
            name,
            destination_type,
            population,
        }) => {
            assert_eq!(member["kind"], "relationship_end");
            node(declaration, "declaration");
            node(destination_type, "destination_type");
            assert_eq!(member["name"], name.as_ref());
            match population {
                Some(value) => node(value, "population"),
                None => assert!(member.get("population").is_none()),
            }
        }
        Some(CheckedOperationMember::Operation { declaration, name }) => {
            assert_eq!(member["kind"], "operation");
            node(declaration, "declaration");
            assert_eq!(member["name"], name.as_ref());
        }
        Some(CheckedOperationMember::TypeArgument { declaration }) => {
            assert_eq!(member["kind"], "type_argument");
            node(declaration, "declaration");
        }
        Some(CheckedOperationMember::ProfileOperator { operator }) => {
            assert_eq!(member["kind"], "profile_operator");
            assert_eq!(operator.as_deref(), member["operator"].as_str());
        }
        Some(CheckedOperationMember::StateClause { clause }) => {
            assert_eq!(member["kind"], "state_clause");
            assert_eq!(member["clause"], clause.as_wire());
        }
        Some(CheckedOperationMember::TemporalInterval { interval }) => {
            assert_eq!(member["kind"], "temporal_interval");
            let expected = if member["interval"].is_null() {
                None
            } else {
                Some(
                    serde_json::from_value::<quire_contract_model::CheckedOperationInterval>(
                        member["interval"].clone(),
                    )
                    .expect("published interval"),
                )
            };
            assert_eq!(interval.as_ref(), expected.as_ref());
        }
        Some(CheckedOperationMember::Fairness {
            fairness_kind,
            granularity,
            target,
        }) => {
            assert_eq!(member["kind"], "fairness");
            assert_eq!(serde_json::to_value(target).unwrap(), member["target"]);
            assert_eq!(
                *fairness_kind,
                serde_json::from_value(member["fairness_kind"].clone()).unwrap()
            );
            assert_eq!(
                *granularity,
                serde_json::from_value(member["granularity"].clone()).unwrap()
            );
        }
    }
}

#[trace("FR-349-AC-6", "FR-349-AC-8", "FR-349-AC-9")]
#[test]
fn qspec_admitted_values_members_and_catalog_constraints_are_exact() {
    let dir = fixtures_dir();
    let proposal = proposal_dir_of(&dir);
    let documents = vector_selected_documents(&proposal).expect("published document selections");
    let document = proposal.join(positive_document(
        "positive-operation-identities.json",
        &documents,
    ));
    let (wire, admitted) = read_positive(&dir, "positive-operation-identities.json", &document)
        .expect("published operation identities admit");
    let mut saw_decimal = false;
    let mut integers = Vec::new();
    let mut saw_indexed_member = false;
    let mut saw_constraints = false;
    for (wire_node, node) in wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("wire nodes")
        .iter()
        .zip(&admitted.graph().nodes)
    {
        if let Some(literal) = node.body.literal_read() {
            assert_eq!(literal.type_id, &node.semantic_type);
            assert_eq!(
                literal.kind.as_wire(),
                wire_node["body"]["value_kind"].as_str().expect("kind")
            );
            match literal.value {
                CheckedLiteralValue::Decimal { coefficient, scale } => {
                    assert_eq!(wire_node["body"]["value"], "1.25");
                    assert_eq!(coefficient.to_string(), "125");
                    assert_eq!(scale, 2);
                    saw_decimal = true;
                }
                CheckedLiteralValue::Integer(value) => {
                    assert_eq!(
                        value.to_string(),
                        wire_node["body"]["value"].as_str().expect("integer")
                    );
                    integers.push(value.to_string());
                }
                _ => {}
            }
        }
        let CheckedTerm::Application { operation, .. } = &node.body else {
            continue;
        };
        let read = operation.read().expect("admitted typed operation");
        if let Some(CheckedOperationMember::Field { declaration, .. }) = read.member {
            saw_indexed_member |= declaration
                .path
                .iter()
                .any(|part| matches!(part, quire_contract_model::CheckedNodePathSegment::Index(_)));
        }
        saw_constraints |= !read.constraints.is_empty();
    }
    assert!(saw_decimal && saw_indexed_member && saw_constraints);
    assert!(integers.contains(&"7".to_owned()) && integers.contains(&"-2".to_owned()));
    let encoded = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            operation_effects: None,
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("encode admitted QSpec package");
    assert_eq!(serde_json::from_slice::<Value>(&encoded).unwrap(), wire);
}

#[trace("FR-349-AC-8", "FR-349-AC-9")]
#[test]
fn qspec_indexed_operation_member_follows_projected_graph_positions() {
    let dir = fixtures_dir();
    let proposal = proposal_dir_of(&dir);
    let documents = vector_selected_documents(&proposal).expect("published document selections");
    let document = proposal.join(positive_document(
        "positive-operation-identities.json",
        &documents,
    ));
    let (_, admitted) = read_positive(&dir, "positive-operation-identities.json", &document)
        .expect("published indexed operation admits");
    let source_node = admitted.graph().nodes.iter().find(|node| matches!(&node.body,
        CheckedTerm::Application { operation, .. } if operation.identity() == Some("quire.op.record.project")))
        .expect("published record projection");
    let source_member = match &source_node.body {
        CheckedTerm::Application { operation, .. } => {
            match operation.read().unwrap().member.unwrap() {
                CheckedOperationMember::Field { declaration, .. } => declaration,
                other => panic!("expected indexed field, got {other:?}"),
            }
        }
        _ => unreachable!(),
    };
    let profile = CompleteLoweringProfileV2 {
        reference: quire_contract_model::LoweringProfileRef {
            authority: "quire".into(),
            identity: "typed-projection-test".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let lowered = admitted.lower(std::slice::from_ref(&source_node.node_id), &profile);
    let producer = lowered.package.wire().expect("projected producer package");
    let projected_wire: Value =
        serde_json::from_slice(lowered.package.canonical_bytes().unwrap()).unwrap();
    let projected_node = projected_wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.record.project")
        .expect("projected operation");
    let expected: CheckedNodeId = serde_json::from_value(
        projected_node["body"]["operation"]["member"]["declaration"].clone(),
    )
    .unwrap();
    let projected_key: CheckedNodeId =
        serde_json::from_value(projected_node["node_id"].clone()).unwrap();
    assert_ne!(
        &expected, source_member,
        "projection must reorder indexed declaration"
    );
    let producer_member = match producer.term(&projected_key).expect("producer term") {
        CheckedTerm::Application { operation, .. } => {
            match operation.read().unwrap().member.unwrap() {
                CheckedOperationMember::Field { declaration, .. } => declaration,
                other => panic!("expected projected field, got {other:?}"),
            }
        }
        other => panic!("expected application, got {other:?}"),
    };
    assert_eq!(producer_member, &expected);
    let ContractPackageReadResult::Admitted(authenticated) =
        ContractPackageWireV1::read_with_source(
            lowered.package.canonical_bytes().unwrap(),
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &admitted,
        )
        .expect("source correspondence")
    else {
        panic!("authenticated projected package must admit")
    };
    let authenticated_member = match authenticated.term(&projected_key).unwrap() {
        CheckedTerm::Application { operation, .. } => {
            match operation.read().unwrap().member.unwrap() {
                CheckedOperationMember::Field { declaration, .. } => declaration,
                other => panic!("expected authenticated field, got {other:?}"),
            }
        }
        _ => panic!("authenticated operation missing"),
    };
    assert_eq!(authenticated_member, &expected);
}

#[trace("TC-048", "FR-038-AC-225")]
#[test]
fn tc_048_qspec_union_values_admit_under_their_application_keys() {
    let (wire, admitted) = read_positive(
        &fixtures_dir(),
        "positive-union-nodes.json",
        &proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT),
    )
    .expect("the current QSpec union fixture admits");
    for (position, member, payload) in [(10, "Rect", vec![8, 9]), (11, "Empty", vec![])] {
        let node = &wire["semantic_graph"]["nodes"][position];
        let retained = &admitted.graph().nodes[position];
        assert_eq!(node["node_tag"], "value");
        assert_eq!(node["semantic_form"], "union_value");
        assert_eq!(
            retained.node_id,
            serde_json::from_value(node["node_id"].clone()).expect("node id")
        );
        assert_eq!(
            serde_json::to_value(&retained.body).expect("retained body"),
            node["body"]
        );
        assert_eq!(node["body"]["term"], "aggregate");
        assert_eq!(node["body"]["members"][0]["name"], member);
        let members = node["body"]["members"][0]["value"]["members"]
            .as_array()
            .expect("payload aggregate");
        assert_eq!(members.len(), payload.len());
        for (actual, referenced) in members.iter().zip(payload) {
            assert_eq!(
                actual["target"],
                wire["semantic_graph"]["nodes"][referenced]["node_id"]
            );
        }
    }
    assert_eq!(
        wire["semantic_graph"]["nodes"][13]["body"]["term"],
        "application"
    );
}

#[trace("TC-048", "FR-038-AC-226")]
#[test]
fn tc_048_qspec_union_values_refuse_duplicate_ids_and_invalid_bodies() {
    let fixtures = fixtures_dir();
    let base = read_json(&fixtures.join("positive-union-nodes.json")).expect("QSpec fixture");
    assert!(matches!(
        read_package(&base),
        Ok(CheckedPackageV2ReadResult::Admitted(_))
    ));
    for position in [10, 11] {
        let mut duplicate = base.clone();
        let node = duplicate["semantic_graph"]["nodes"][position].clone();
        let later = duplicate["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        duplicate["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&duplicate).expect("reader result")
        else {
            panic!("duplicate structural id of node {position} admitted");
        };
        assert_eq!(refusal.code, Code::InvalidSemanticGraph);
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{later}/node_id"))
        );
    }
    for position in [10, 11] {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][position]["body"]["members"][0]["name"] =
            json!("NoSuchCase");
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&changed).expect("reader result")
        else {
            panic!("invalid union case at node {position} admitted");
        };
        assert_eq!(refusal.code, Code::IllTyped);
        assert_eq!(refusal.cause, Some(Cause::TypeMismatch));
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{position}"))
        );
    }
    let mut missing = base;
    missing["semantic_graph"]["nodes"][10]["body"]["members"][0]["value"]["members"][0]["target"] =
        json!({"package":"example","path":["Missing"],"ordinal":0});
    fixture_support::settle(&mut missing);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&missing).expect("reader result")
    else {
        panic!("unresolved union payload reference admitted");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph);
    assert_eq!(refusal.cause, None);
    assert!(
        refusal.loci[0]
            .path
            .as_ref()
            .expect("reference refusal path")
            .to_string()
            .starts_with("/semantic_graph/nodes/10/body"),
        "{refusal:?}"
    );
}

/// Read one QSpec positive package with the selected domain document, if any.
/// The document is supplied under the selected package identity; the
/// production reader checks that it names that identity.
fn read_positive(
    fixtures: &Path,
    name: &str,
    domain_document: &Path,
) -> Result<(Value, CheckedPackageV2), String> {
    let package = read_json(&fixtures.join(name))?;
    read_positive_package(package, name, domain_document)
}

/// Admit a published positive package or a single-input adverse mutation of it.
fn read_positive_package(
    package: Value,
    name: &str,
    domain_document: &Path,
) -> Result<(Value, CheckedPackageV2), String> {
    match read_package_with_document(&package, name, domain_document)? {
        CheckedPackageV2ReadResult::Admitted(admitted) => Ok((package, *admitted)),
        other => Err(format!("{name}: positive package did not admit: {other:?}")),
    }
}

#[trace("TC-048", "FR-038-AC-176")]
#[test]
fn tc_048_qspec_all_positive_packages_and_owner_identities() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let documents = vector_selected_documents(&proposal).expect("published document selections");
    let mut owners = std::collections::BTreeMap::new();
    for name in POSITIVE_ALL {
        let document = proposal.join(positive_document(name, &documents));
        let (wire, admitted) =
            read_positive(&fixtures, name, &document).unwrap_or_else(|why| panic!("{why}"));
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("published graph nodes");
        assert_eq!(
            admitted.graph().nodes.len(),
            nodes.len(),
            "{name}: graph length"
        );
        assert_eq!(
            admitted.name(),
            wire["name"].as_str().expect("package name")
        );
        for (node, published) in admitted.graph().nodes.iter().zip(nodes) {
            let id = serde_json::to_value(&node.node_id).expect("typed node id");
            let owner = serde_json::to_value(&node.owner).expect("typed owner");
            assert_eq!(id, published["node_id"], "{name}: structural node id");
            assert_eq!(owner, published["owner"], "{name}: graph owner");
            assert_eq!(
                serde_json::to_value(&node.body).expect("retained body"),
                published["body"],
                "{name}: complete body"
            );
            if name.starts_with("positive-two-owners-") {
                for declaration in ["Point", "List"] {
                    if published["declaration"]["qualified_name"] == json!([declaration]) {
                        owners.insert((name, declaration), id.clone());
                        assert_eq!(owner["kind"], "source", "{name}: {declaration} owner");
                    }
                }
            }
        }
    }
    for declaration in ["Point", "List"] {
        let a = owners.get(&("positive-two-owners-a.json", declaration));
        let b = owners.get(&("positive-two-owners-b.json", declaration));
        assert!(
            a.is_some() && b.is_some(),
            "{declaration}: both source owners must occur"
        );
        assert_ne!(
            a, b,
            "{declaration}: distinct source owners need distinct node ids"
        );
    }
}

/// QSpec's nominal declarations and derived nodes have assigned identities;
/// neither body bytes nor owner spelling participates in their construction.
#[trace("TC-048", "TC-228", "FR-038-AC-89")]
#[test]
fn tc_048_published_nominal_structural_ids_survive_reader_and_writer() {
    let fixtures = fixtures_dir();
    let name = "positive-nominal-identities.json";
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    let (wire, admitted) =
        read_positive(&fixtures, name, &document).unwrap_or_else(|why| panic!("{why}"));
    let declared = |path: &[&str]| json!({"package":"example", "path":path, "ordinal":0});
    let status = declared(&["Example", "Status"]);
    let length = declared(&["Example", "Length"]);
    let distance = declared(&["Example", "Distance"]);
    let expected = [
        json!({"package":"example", "path":["value", "enum_value", status, "READY"], "ordinal":0}),
        status,
        declared(&["Example", "metre"]),
        length.clone(),
        distance.clone(),
        json!({"package":"example", "path":["scalar_type", "dimension_equivalence", distance, length], "ordinal":0}),
    ];
    let published = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nominal nodes");
    assert_eq!(published.len(), expected.len());
    for (position, (node, id)) in admitted.graph().nodes.iter().zip(expected).enumerate() {
        assert_eq!(
            serde_json::to_value(&node.node_id).expect("node id"),
            published[position]["node_id"],
            "{position}: retained assigned wire id"
        );
        assert_eq!(
            serde_json::to_value(node_index::fixture_structural_key(
                &admitted.graph().nodes,
                position
            ))
            .expect("resolved structural id"),
            id,
            "{position}: resolved structural id"
        );
        assert_eq!(
            serde_json::to_value(&node.owner).expect("owner"),
            published[position]["owner"],
            "{position}: retained owner"
        );
        assert_eq!(
            serde_json::to_value(&node.body).expect("body"),
            published[position]["body"],
            "{position}: retained complete body"
        );
    }
    let encoded = quire_contract_model::encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            operation_effects: None,
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("published package encodes");
    assert_eq!(
        encoded,
        quire_canonical::to_vec(&wire, quire_canonical::Limits::new(1 << 24))
            .expect("published canonical bytes")
    );
}

#[trace("TC-048", "FR-038-AC-176")]
#[test]
fn tc_048_qspec_positive_inputs_fail_closed() {
    let fixtures = fixtures_dir();
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    let missing = read_positive(&fixtures, "positive-absent.json", &document)
        .expect_err("a required fixture must exist");
    assert!(missing.contains("positive-absent.json"), "{missing}");
    let base = "positive-all-families.json";
    let missing_document = read_positive(&fixtures, base, &fixtures.join("absent-domain.json"))
        .expect_err("the selected document must exist");
    assert!(
        missing_document.contains("absent-domain.json"),
        "{missing_document}"
    );
    let scratch = std::env::temp_dir().join(format!("ir-654-domain-{}", std::process::id()));
    std::fs::write(&scratch, b"not json").expect("write malformed domain document");
    let malformed = read_positive(&fixtures, base, &scratch)
        .expect_err("a malformed selected document must fail");
    std::fs::remove_file(&scratch).expect("remove malformed domain document");
    assert!(
        malformed.contains(&scratch.display().to_string()),
        "{malformed}"
    );
    assert!(malformed.contains("is not JSON"), "{malformed}");
    let package = read_json(&fixtures.join(base)).expect("published positive package");
    let mut unknown = package.clone();
    let root_position = unknown["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "model_root")
        .expect("selected model root");
    for node in unknown["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["semantic_form"] == "model_root" {
            node["owner"]["identity"] = json!("acme/unknown");
        }
    }
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package_with_document(&unknown, base, &document).expect("unknown selected model read")
    else {
        panic!("unknown selected model must refuse");
    };
    assert_eq!(refusal.code, Code::MissingImport);
    assert_eq!(refusal.cause, Some(Cause::MissingSelection));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{root_position}/owner/identity"
        ))
    );
    for retired in ["package_id", "identity_preimage", "identity_projection"] {
        let mut changed = package.clone();
        changed[retired] = json!({});
        let refused = read_positive_package(changed, base, &document)
            .expect_err("retired identity member must fail");
        assert!(refused.contains(base), "{refused}");
        assert!(refused.contains("UnknownMember"), "{refused}");
    }
}

#[trace("TC-048", "FR-038-AC-310", "FR-038-AC-363")]
#[test]
fn tc_048_package_name_is_required_and_retired_identity_members_refuse() {
    let mut package = read_json(&fixtures_dir().join(MUTATED_BASE)).expect("QSpec fixture");
    let admitted = read_package(&package).expect("production read");
    let CheckedPackageV2ReadResult::Admitted(admitted) = admitted else {
        panic!("published package must admit: {admitted:?}");
    };
    assert_eq!(admitted.name(), package["name"].as_str().expect("name"));

    for retired in ["package_id", "identity_preimage", "identity_projection"] {
        package[retired] = json!({});
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&package).expect("production read")
        else {
            panic!("retired top-level {retired} admitted");
        };
        assert_eq!(refusal.code, Code::UnknownMember, "{retired}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/{retired}"))
        );
        package.as_object_mut().expect("package").remove(retired);
        assert!(matches!(
            read_package(&package).expect("production read"),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
    }

    package.as_object_mut().expect("package").remove("name");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("production read")
    else {
        panic!("missing package name admitted");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/name".to_owned())
    );
}

#[trace("FR-038-AC-145")]
#[test]
fn tc_226_qspec_recursive_records_admit() {
    let path = fixtures_dir().join(RECURSIVE);
    let package = read_json(&path).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(reading_of(&package).expect("fixture reading"), "admitted");
}

/// The base package and the adverse mutations of the named checkout; a test
/// failure when either cannot be read.
fn adverse_inputs() -> (Value, Value) {
    let dir = fixtures_dir();
    let base = read_json(&dir.join(MUTATED_BASE)).unwrap_or_else(|why| panic!("{why}"));
    let adverse = read_json(&dir.join(ADVERSE)).unwrap_or_else(|why| panic!("{why}"));
    (base, adverse)
}

/// Fails the test with every problem, or passes when there are none.
fn assert_no_problems(problems: &[String]) {
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Every mutation refuses as recorded, with the expected-failure list empty
/// (FR-038-AC-118): each of the five `body_grammar_mutations` refuses
/// `malformed_wire`, with no identity refreshed.
#[trace("TC-048", "FR-038-AC-112", "FR-038-AC-118", "FR-038-AC-133")]
#[test]
fn tc_048_qspec_adverse_mutations_refuse_as_recorded() {
    let (base, adverse) = adverse_inputs();
    assert_no_problems(&adverse_problems(&base, &adverse, EXPECTED_FAILURES));
    let grammar = adverse["body_grammar_mutations"].as_array().expect("list");
    let ids = grammar
        .iter()
        .map(|entry| entry["id"].as_str().expect("id"))
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [
            "application-in-application-arguments",
            "application-in-aggregate-members",
            "application-in-binding-value",
            "aggregate-in-group-members",
            "binding-as-body-root",
        ]
    );
    for entry in grammar {
        assert_eq!(entry["outcome"], "refused:malformed_wire", "{entry}");
    }
    // FR-038-AC-118: none of the five is a listed exception, so each refuses as
    // recorded, which `adverse_problems` has just checked of every unlisted one.
    for listed in EXPECTED_FAILURES {
        assert!(
            !ids.contains(&listed.id),
            "{} is a body-grammar mutation, which refuses as recorded",
            listed.id
        );
    }
}

/// Whether some problem contains `text`.
fn mentions(problems: &[String], text: &str) -> bool {
    problems.iter().any(|problem| problem.contains(text))
}

/// A recorded outcome names the cause only where it gives one: a cause-less
/// outcome is met by the code with any cause, and by no other code.
#[trace("TC-048", "FR-038-AC-112")]
#[test]
fn tc_048_a_recorded_outcome_without_a_cause_is_met_by_its_code() {
    let code = "refused:invalid_package";
    assert!(refuses_as(code, code));
    assert!(refuses_as("refused:invalid_package/invalid-value", code));
    assert!(!refuses_as("refused:ill_typed/invalid-value", code));
    assert!(!refuses_as("refused:invalid_package_other", code));
    assert!(!refuses_as("admitted", code));
    // A recorded cause is compared.
    let caused = "refused:invalid_package/invalid-value";
    assert!(refuses_as(caused, caused));
    assert!(!refuses_as(
        "refused:invalid_package/unknown-operation",
        caused
    ));
    assert!(!refuses_as(code, caused));
}

#[trace("TC-048", "FR-038-AC-112")]
#[test]
fn tc_048_qspec_adverse_run_fails_closed() {
    // The checkout: unset, empty and a path without the fixtures all fail.
    assert!(fixtures_dir_of(None).is_err());
    assert!(fixtures_dir_of(Some("")).is_err());
    assert!(fixtures_dir_of(Some("/nonexistent/quire-specification")).is_err());
    // The file: missing and not JSON both fail.
    let dir = fixtures_dir();
    assert!(read_json(&dir.join("no-such-adverse.json")).is_err());
    let scratch = std::env::temp_dir().join(format!("conformance-qspec-{}", std::process::id()));
    std::fs::write(&scratch, b"not json").expect("the scratch file is written");
    let unreadable = read_json(&scratch);
    std::fs::remove_file(&scratch).expect("the scratch file is removed");
    assert!(unreadable.is_err());

    let (base, adverse) = adverse_inputs();
    let versions = adverse["structural_mutations"]
        .as_array()
        .and_then(|list| {
            list.iter()
                .find(|entry| entry["id"] == "unknown-contract-version")
        })
        .expect("the file holds unknown-contract-version")
        .clone();
    let body = adverse["body_grammar_mutations"][0].clone();
    let with = |structural: Value, grammar: Value| json!({"structural_mutations": [structural], "body_grammar_mutations": [grammar]});

    // A list absent or empty fails, by its own problem: the other list holds
    // only the entry the reader refuses as recorded, so nothing else fails.
    let no_list = |name: &str| format!("{ADVERSE} has no non-empty {name} list");
    let absent_body = adverse_problems(
        &base,
        &json!({"structural_mutations": [versions.clone()]}),
        &[],
    );
    assert_eq!(absent_body, [no_list("body_grammar_mutations")]);
    let empty_body = adverse_problems(
        &base,
        &json!({"structural_mutations": [versions.clone()], "body_grammar_mutations": []}),
        &[],
    );
    assert_eq!(empty_body, [no_list("body_grammar_mutations")]);
    let absent_structural = adverse_problems(
        &base,
        &json!({"body_grammar_mutations": [body.clone()]}),
        &[],
    );
    assert!(mentions(
        &absent_structural,
        &no_list("structural_mutations")
    ));
    let empty_structural = adverse_problems(
        &base,
        &json!({"structural_mutations": [], "body_grammar_mutations": [body.clone()]}),
        &[],
    );
    assert!(mentions(
        &empty_structural,
        &no_list("structural_mutations")
    ));
    // A body-grammar entry lacking `flattened` fails, whatever the reader gives it.
    let mut unflattened = body.clone();
    unflattened
        .as_object_mut()
        .expect("an entry")
        .remove("flattened");
    let problems = adverse_problems(&base, &with(versions.clone(), unflattened), &[]);
    assert!(
        problems.iter().any(|problem| problem.contains("flattened")),
        "{problems:?}"
    );
    // An unlisted mutation the reader does not refuse as recorded fails.
    let mut misrecorded = versions.clone();
    misrecorded["outcome"] = json!("refused:malformed_wire");
    let sound = {
        let mut sound = body.clone();
        sound["outcome"] = json!("refused:malformed_wire");
        sound
    };
    let problems = adverse_problems(&base, &with(misrecorded.clone(), sound.clone()), &[]);
    assert!(
        problems
            .iter()
            .any(|problem| problem.starts_with("unknown-contract-version:")),
        "{problems:?}"
    );
    // A listed id absent from the file, one the reader refuses as recorded and
    // one it refuses with neither the recorded nor the listed code all fail.
    let entry = |id, refusal| ExpectedFailure {
        id,
        refusal,
        ticket: "IR-0",
    };
    let absent = adverse_problems(&base, &adverse, &[entry("no-such-mutation", "x")]);
    assert!(
        absent.iter().any(|problem| problem.contains("absent from")),
        "{absent:?}"
    );
    let stale = adverse_problems(
        &base,
        &adverse,
        &[entry("unknown-contract-version", "refused:invalid_package")],
    );
    assert!(
        stale
            .iter()
            .any(|problem| problem.contains("delete the entry")),
        "{stale:?}"
    );
    let neither = adverse_problems(
        &base,
        &with(misrecorded, sound),
        &[entry("unknown-contract-version", "refused:invalid_package")],
    );
    assert!(
        neither.iter().any(|problem| problem.contains("neither")),
        "{neither:?}"
    );
}
