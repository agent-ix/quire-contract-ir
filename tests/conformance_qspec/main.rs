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
    CheckedArtifactRef, CheckedExtensionClauseSchema, CheckedExtensionDefinition,
    CheckedExtensionGrammarSchema, CheckedExtensionNodeSchema, CheckedExtensionProduction,
    CheckedExtensionTypedNodeSchema, CheckedNodeId, CheckedPackageEvidence,
    CheckedPackageEvidenceInsertionError, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

mod dimension_equivalence;
mod extension;
#[path = "../it/support/checked_package.rs"]
mod fixture_support;
mod hybrid;
mod model_root;
mod model_tuple_union;
mod reaches_population;
mod selection_vectors;
mod single_body;
mod timed_model;
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
        let document = std::fs::read(&selected_document)
            .map_err(|error| format!("{} cannot be read: {error}", selected_document.display()))?;
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
    ids.sort_by(|left, right| typed_node_id(left).cmp(&typed_node_id(right)));
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

#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn tc_048_qspec_positive_fixtures_admit() {
    let dir = fixtures_dir();
    let document = proposal_dir_of(&dir).join(DOMAIN_DOCUMENT);
    for name in POSITIVE_ALL {
        let (package, admitted) =
            read_positive(&dir, name, &document).unwrap_or_else(|why| panic!("{why}"));
        let _ = (&package, &admitted);
    }
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
        assert_eq!(refusal.code, Code::InvalidPackage);
        assert!(
            refusal.loci[0]
                .path
                .as_ref()
                .expect("body refusal path")
                .to_string()
                .starts_with(&format!("/semantic_graph/nodes/{position}/body")),
            "{refusal:?}"
        );
    }
    let mut missing = base;
    missing["semantic_graph"]["nodes"][10]["body"]["members"][0]["value"]["members"][0]["target"] =
        json!({"package":"example","path":["Missing"],"ordinal":0});
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&missing).expect("reader result")
    else {
        panic!("unresolved union payload reference admitted");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration);
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
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    let mut owners = std::collections::BTreeMap::new();
    for name in POSITIVE_ALL {
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
            published[position]["node_id"], id,
            "{position}: assigned wire id"
        );
        assert_eq!(serde_json::to_value(&node.node_id).expect("node id"), id);
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
    for node in unknown["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["semantic_form"] == "model_root" {
            node["owner"]["identity"] = json!("acme/unknown");
        }
    }
    let refused = read_positive_package(unknown, base, &document)
        .expect_err("unknown selected model must fail");
    assert!(refused.contains("acme/unknown"), "{refused}");
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

/// Apply a QSpec derived-shape mutation to every matching node in its positive package,
/// preserving the structural node id. The fixture and mutation are always
/// read from the QSpec checkout.
fn derived_shape_mutations(dir: &Path, row: &Value) -> Result<Vec<(usize, Value)>, String> {
    let base = text_of(row, "base")?;
    let original = read_json(&dir.join(format!("{base}.json")))?;
    let tag = row["node"]["node_tag"]
        .as_str()
        .ok_or("mutation has no node_tag")?;
    let form = row["node"]["semantic_form"]
        .as_str()
        .ok_or("mutation has no semantic_form")?;
    let nodes = original["semantic_graph"]["nodes"]
        .as_array()
        .ok_or("semantic_graph.nodes is not an array")?;
    let matches = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node["node_tag"] == tag && node["semantic_form"] == form)
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return Err(format!("{base}: no {tag}/{form} node"));
    }
    let patches = row["patch"]
        .as_array()
        .ok_or("mutation has no patch array")?;
    matches
        .into_iter()
        .map(|position| {
            let mut package = original.clone();
            for patch in patches {
                if text_of(patch, "op")? != "replace" {
                    return Err("mutation uses an unsupported patch op".into());
                }
                let path = text_of(patch, "path")?;
                let replacement = patch.get("value").ok_or("patch has no value")?;
                replace_at(
                    &mut package,
                    &format!("/semantic_graph/nodes/{position}{path}"),
                    replacement,
                )?;
            }
            Ok((position, package))
        })
        .collect()
}

#[trace("FR-038-AC-123", "FR-038-AC-133")]
#[test]
fn tc_226_qspec_ungrouped_derived_shape_mutations_refuse() {
    let dir = fixtures_dir();
    let adverse = read_json(&dir.join(ADVERSE)).expect("QSpec adverse fixture");
    let rows = mutations(&adverse, "derived_shape_key_mutations").expect("mutation rows");
    for row in rows.iter().filter(|row| {
        row["base"] != "positive-recursive-records" && row["node"]["semantic_form"] != "record"
    }) {
        let id = text_of(row, "id").expect("mutation id");
        let packages =
            derived_shape_mutations(&dir, row).unwrap_or_else(|why| panic!("{id}: {why}"));
        let expected = format!(
            "refused:{}/{}",
            text_of(row, "expected_code").expect("expected code"),
            text_of(row, "expected_cause").expect("expected cause")
        );
        for (position, package) in packages {
            assert_eq!(
                reading_of(&package).expect("package reading"),
                expected,
                "{id}: node {position}"
            );
        }
    }
}

/// IR-630, after QSL-638 supplies the declared SourceOwner on the wire.
#[trace("FR-038-AC-150")]
#[test]
#[ignore = "IR-630: declared SourceOwner is absent from the v2 semantic node wire"]
fn tc_226_qspec_grouped_and_declared_derived_shape_mutations_refuse() {
    let dir = fixtures_dir();
    let adverse = read_json(&dir.join(ADVERSE)).expect("QSpec adverse fixture");
    let rows = mutations(&adverse, "derived_shape_key_mutations").expect("mutation rows");
    for row in rows.iter().filter(|row| {
        row["base"] == "positive-recursive-records" || row["node"]["semantic_form"] == "record"
    }) {
        let id = text_of(row, "id").expect("mutation id");
        let packages =
            derived_shape_mutations(&dir, row).unwrap_or_else(|why| panic!("{id}: {why}"));
        let expected = format!(
            "refused:{}/{}",
            text_of(row, "expected_code").expect("expected code"),
            text_of(row, "expected_cause").expect("expected cause")
        );
        for (position, package) in packages {
            assert_eq!(
                reading_of(&package).expect("package reading"),
                expected,
                "{id}: node {position}"
            );
        }
    }
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
#[trace("TC-048", "FR-038-AC-112", "FR-038-AC-118")]
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
