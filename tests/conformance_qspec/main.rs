// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-107, AC-112, AC-113, AC-176 and AC-310: QSpec's CheckedPackage V2 fixtures and
//! vectors against the production reader.
//!
//! AC-107: the positive fixtures admit end to end. AC-112: every mutation of
//! `adverse.json` refuses as recorded, with no identity refreshed, bar the
//! listed expected failures, and each body-grammar `flattened` form is not
//! refused `malformed_wire`. AC-113: the version-free dependency selections of
//! `dependency-selection-vectors.json` derive the recorded `package_id`
//! through the reader's own derivation. AC-310: shipped identity projections
//! and other extra selection members refuse `malformed_wire` at their exact
//! pointers, while the single-body positives retain their recorded identity.
//!
//! The fixtures are QSpec's and are read from a `quire-specification` checkout
//! named by `QUIRE_SPECIFICATION_DIR`; nothing of them is copied into this
//! repository. This target is outside the run of `make test` (its manifest
//! entry has `test = false`, and a plain `make test` has no checkout) and is
//! run by `make conformance-qspec` alone. It fails, and never skips, when the
//! variable is unset or empty, when it names a path that does not hold
//! `proposals/checked-package-v2/fixtures/`, when a file cannot be read or is
//! not JSON, when a list is absent or empty, and when a result is not the
//! recorded one.

use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    CheckedArtifactRef, CheckedExtensionClauseSchema, CheckedExtensionDefinition,
    CheckedExtensionGrammarSchema, CheckedExtensionNodeSchema, CheckedExtensionProduction,
    CheckedExtensionTypedNodeSchema, CheckedPackageEvidence, CheckedPackageEvidenceInsertionError,
    CheckedPackageIdentityPreimageRefV2, CheckedPackageIdentityPreimageV2, CheckedPackageLockV2,
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedSemanticGraphV2,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

mod extension;
#[path = "../it/support/checked_package.rs"]
mod fixture_support;
mod hybrid;
mod reaches_population;
mod single_body;
mod timed_model;

/// The environment variable naming the root of a `quire-specification` checkout.
const SPECIFICATION_DIR: &str = "QUIRE_SPECIFICATION_DIR";

/// Where QSpec publishes the CheckedPackage V2 fixtures, from its root.
const FIXTURES: &str = "proposals/checked-package-v2/fixtures";

/// The three positive fixtures FR-038-AC-107 reads.
const POSITIVE: [&str; 3] = [
    "positive-all-families.json",
    "positive-clause-operations.json",
    "positive-union-nodes.json",
];

/// The complete published positive set required by FR-038-AC-176.
const POSITIVE_ALL: [&str; 9] = [
    "positive-all-families.json",
    "positive-clause-operations.json",
    "positive-control-operations.json",
    "positive-nominal-identities.json",
    "positive-operation-identities.json",
    "positive-recursive-records.json",
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

/// The dependency-selection vectors FR-038-AC-113 reads, under the proposal.
const SELECTION_VECTORS: &str = "dependency-selection-vectors.json";

/// The byte ceiling of the package-id derivation: far above any fixture.
const DERIVATION_CEILING: u64 = 1 << 24;

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
    let selections = package["lock"]["model_selections"]
        .as_array()
        .ok_or_else(|| format!("{name}: lock.model_selections is not an array"))?;
    for selection in selections {
        let identity = selection["identity"]
            .as_str()
            .ok_or_else(|| format!("{name}: model selection has no identity"))?;
        if identity != "acme/orders" {
            return Err(format!(
                "{name}: unknown selected model identity {identity}"
            ));
        }
        let digest = selection["digest"]
            .as_str()
            .filter(|digest| !digest.is_empty())
            .ok_or_else(|| format!("{name}: model selection {identity} has no digest"))?;
        let document = std::fs::read(domain_document)
            .map_err(|error| format!("{} cannot be read: {error}", domain_document.display()))?;
        serde_json::from_slice::<Value>(&document)
            .map_err(|error| format!("{} is not JSON: {error}", domain_document.display()))?;
        evidence
            .insert_domain_package_document(digest, document)
            .map_err(|error| format!("{name}: evidence insertion failed: {error}"))?;
    }
    Ok(CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ))
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
        Code::StaleDependency => "stale_dependency",
        Code::DigestDomainMismatch => "digest_domain_mismatch",
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
        Cause::ContentMismatch => "content-mismatch",
        Cause::MissingName => "missing-name",
        Cause::MalformedDeclaration => "malformed-declaration",
        Cause::StaleNodeKey => "stale-node-key",
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
        Cause::DigestDomainMismatch => "digest-domain-mismatch",
        Cause::ByteDigestMismatch => "byte-digest-mismatch",
        Cause::WrongModelSelection => "wrong-model-selection",
        Cause::ConflictingBinding => "conflicting-binding",
        Cause::UnpreservedModelMeaning => "unpreserved-model-meaning",
        Cause::ConflictingDefinition => "conflicting-definition",
        Cause::InvalidValue => "invalid-value",
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

/// The `package_id` the reader's own derivation gives `package` with
/// `selections` as the `dependency_selections` of its lock and identity
/// preimage, as the hex digest.
fn derived_with(package: &Value, selections: Option<&Value>) -> Result<String, String> {
    let mut package = package.clone();
    if let Some(selections) = selections {
        replace_at(&mut package, "/lock/dependency_selections", selections)?;
        replace_at(
            &mut package,
            "/identity_preimage/dependency_selections",
            selections,
        )?;
    }
    let lock: CheckedPackageLockV2 = serde_json::from_value(package["lock"].clone())
        .map_err(|error| format!("the lock is not a V2 lock: {error}"))?;
    let preimage: CheckedPackageIdentityPreimageV2 =
        serde_json::from_value(package["identity_preimage"].clone())
            .map_err(|error| format!("the identity preimage is not a V2 preimage: {error}"))?;
    if lock.dependency_selections != preimage.dependency_selections {
        return Err("the lock and the preimage hold different dependency_selections".into());
    }
    let graph: CheckedSemanticGraphV2 =
        serde_json::from_value(package["semantic_graph"].clone())
            .map_err(|error| format!("the graph is not a V2 graph: {error}"))?;
    let input = CheckedPackageIdentityPreimageRefV2 {
        version: &preimage.version,
        edition: &preimage.edition,
        profile_selections: &preimage.profile_selections,
        definition_selections: &preimage.definition_selections,
        model_selections: &preimage.model_selections,
        required_features: &preimage.required_features,
        dependency_selections: &preimage.dependency_selections,
        identity_projection: &graph.nodes,
    };
    quire_canonical::sha256(&input, Limits::new(DERIVATION_CEILING))
        .map(|digest| digest.to_string())
        .map_err(|error| format!("the preimage does not encode: {error:?}"))
}

/// FR-038-AC-113 over the vectors file: every problem found, empty when the
/// run passes. `base` is the package the vectors name.
fn selection_problems(base: &Value, vectors: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let Some(recorded) = vectors.get("package_id").and_then(Value::as_str) else {
        return vec![format!("{SELECTION_VECTORS} has no string package_id")];
    };
    let Some(selections) = vectors
        .get("dependency_selections")
        .filter(|value| value.as_array().is_some_and(|entries| !entries.is_empty()))
    else {
        return vec![format!(
            "{SELECTION_VECTORS} has no non-empty dependency_selections"
        )];
    };
    match derived_with(base, Some(selections)) {
        Ok(derived) if derived == recorded => {}
        Ok(derived) => problems.push(format!("derived {derived}, not the recorded {recorded}")),
        Err(why) => problems.push(why),
    }
    let own = base["package_id"]["digest"].as_str();
    match derived_with(base, None) {
        Ok(unchanged) if unchanged == recorded => {
            problems.push("the unchanged base derives the recorded package_id too".into());
        }
        Ok(unchanged) if Some(unchanged.as_str()) != own => problems.push(format!(
            "the unchanged base derives {unchanged}, not its own recorded package_id {own:?}"
        )),
        Ok(_) => {}
        Err(why) => problems.push(why),
    }
    problems
}

/// The path, from the proposal directory, of the package the vectors name, or
/// why they name none.
fn base_path_of(vectors: &Value) -> Result<&str, String> {
    vectors
        .get("base")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| format!("{SELECTION_VECTORS} has no string base"))
}

#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn tc_048_qspec_positive_fixtures_admit() {
    let dir = fixtures_dir();
    let document = proposal_dir_of(&dir).join(DOMAIN_DOCUMENT);
    for name in POSITIVE {
        let (package, admitted) =
            read_positive(&dir, name, &document).unwrap_or_else(|why| panic!("{why}"));
        let derived = serde_json::to_value(admitted.package_id()).expect("a package id serializes");
        assert_eq!(
            derived, package["package_id"],
            "{name}: recorded package_id"
        );
    }
}

fn union_value_preimage(node: &Value) -> Value {
    json!({
        "version": "quire.application-node/v1",
        "node_tag": node["node_tag"],
        "semantic_form": node["semantic_form"],
        "semantic_type": node["semantic_type"],
        "declaration": node.get("declaration"),
        "recursion": null,
        "body": node["body"],
    })
}

fn canonical_digest(value: &Value) -> String {
    quire_canonical::sha256(value, Limits::new(DERIVATION_CEILING))
        .expect("test-authored preimage encodes")
        .to_string()
}

/// Replaces one selected digest in a published, shallow JSON package. The
/// fixture is read from QSpec; the test retains no copy of its bytes.
fn rewrite_digest(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(text) if text == old => *text = new.to_owned(),
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| rewrite_digest(item, old, new)),
        Value::Object(members) => members
            .values_mut()
            .for_each(|member| rewrite_digest(member, old, new)),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
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
    assert_eq!(
        admitted.package_id().digest.as_ref(),
        "e28d31bb3c8f5e3f83f7f660dccb092455963b56c3137c2aa1a5c9d410dda9ad"
    );
    for (position, member, fixed_key, payload) in [
        (
            10,
            "Rect",
            "8dfdf62acb13476b3aa64f7b414f19b571e81e8aa89ce93dd453e2c8e22728f1",
            vec![8, 9],
        ),
        (
            11,
            "Empty",
            "a6e41bcd8c595ae730b9f214fbe0b5496ffff2c63d35e92b52f8b745ddf27211",
            vec![],
        ),
    ] {
        let node = &wire["semantic_graph"]["nodes"][position];
        let retained = &admitted.graph().nodes[position];
        assert_eq!(node["node_tag"], "value");
        assert_eq!(node["semantic_form"], "union_value");
        assert_eq!(node["node_id"]["digest"], fixed_key);
        assert_eq!(retained.node_id.digest.as_ref(), fixed_key);
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
        assert_eq!(canonical_digest(&union_value_preimage(node)), fixed_key);
    }
    assert_eq!(
        wire["semantic_graph"]["nodes"][13]["body"]["term"],
        "application"
    );
}

#[trace("TC-048", "FR-038-AC-226")]
#[test]
fn tc_048_qspec_union_values_refuse_retained_structural_keys_and_body_changes() {
    let fixtures = fixtures_dir();
    let base = read_json(&fixtures.join("positive-union-nodes.json")).expect("QSpec fixture");
    assert!(matches!(
        read_package(&base),
        Ok(CheckedPackageV2ReadResult::Admitted(_))
    ));
    for position in [10, 11] {
        let mut stale = base.clone();
        let node = &stale["semantic_graph"]["nodes"][position];
        let fresh = node["node_id"]["digest"]
            .as_str()
            .expect("fresh key")
            .to_owned();
        let old_preimage = json!({
            "version": "quire.structural-node/v1",
            "node_tag": node["node_tag"],
            "semantic_form": node["semantic_form"],
            "semantic_type": node["semantic_type"],
            "declaration": null,
            "recursion": null,
            "body": node["body"],
        });
        let old = canonical_digest(&old_preimage);
        assert_ne!(old, fresh);
        rewrite_digest(&mut stale, &fresh, &old);
        stale["package_id"]["digest"] = json!(derived_with(&stale, None).expect("package id"));
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&stale).expect("reader result")
        else {
            panic!("retained structural key of node {position} admitted");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::StaleNodeKey))
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(format!("/semantic_graph/nodes/{position}/node_id").as_str())
        );
        assert_eq!(
            refusal.locus.as_ref().map(|id| id.digest.as_ref()),
            Some(old.as_str())
        );
        assert_eq!(
            refusal.expected_node_id().map(|id| id.digest.as_ref()),
            Some(fresh.as_str())
        );
    }
    for position in [10, 11] {
        let mut changed = base.clone();
        if position == 10 {
            changed["semantic_graph"]["nodes"][position]["body"]["members"][0]["value"]["members"]
                .as_array_mut()
                .expect("Rect payload")
                .swap(0, 1);
        } else {
            changed["semantic_graph"]["nodes"][position]["body"]["members"][0]["name"] =
                json!("Rect");
        }
        changed["package_id"]["digest"] = json!(derived_with(&changed, None).expect("package id"));
        let expected = canonical_digest(&union_value_preimage(
            &changed["semantic_graph"]["nodes"][position],
        ));
        let retained = base["semantic_graph"]["nodes"][position]["node_id"]["digest"]
            .as_str()
            .expect("retained key");
        assert_ne!(expected, retained);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&changed).expect("reader result")
        else {
            panic!("body mutation of node {position} with a retained key admitted");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::StaleNodeKey))
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(format!("/semantic_graph/nodes/{position}/node_id").as_str())
        );
        assert_eq!(
            refusal.locus.as_ref().map(|id| id.digest.as_ref()),
            Some(retained)
        );
        assert_eq!(
            refusal.expected_node_id().map(|id| id.digest.as_ref()),
            Some(expected.as_str())
        );
    }
}

/// Read one QSpec positive package with the selected domain document, if any.
/// The document is supplied under the selection's own digest; the production
/// reader verifies that claim against its bytes.
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
        assert_eq!(
            serde_json::to_value(admitted.package_id()).expect("typed package id"),
            wire["package_id"],
            "{name}: published package_id"
        );
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("published graph nodes");
        assert_eq!(
            admitted.graph().nodes.len(),
            nodes.len(),
            "{name}: graph length"
        );
        for (node, published) in admitted.graph().nodes.iter().zip(nodes) {
            let id = serde_json::to_value(&node.node_id).expect("typed node id");
            let owner = serde_json::to_value(&node.owner).expect("typed owner");
            assert_eq!(id, published["node_id"], "{name}: graph node id");
            assert_eq!(owner, published["owner"], "{name}: graph owner");
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
    unknown["lock"]["model_selections"][0]["identity"] = json!("acme/unknown");
    let refused = read_positive_package(unknown, base, &document)
        .expect_err("unknown selected model must fail");
    assert!(refused.contains("acme/unknown"), "{refused}");
    let mut absent_digest = package.clone();
    absent_digest["lock"]["model_selections"][0]
        .as_object_mut()
        .expect("published selection")
        .remove("digest");
    let refused = read_positive_package(absent_digest, base, &document)
        .expect_err("selection without digest must fail");
    assert!(refused.contains("acme/orders"), "{refused}");
    assert!(refused.contains("no digest"), "{refused}");
    let mut changed_package_id = package;
    changed_package_id["package_id"]["digest"] = json!("0".repeat(64));
    let refused = read_positive_package(changed_package_id, base, &document)
        .expect_err("non-admitted positive package must fail");
    assert!(refused.contains(base), "{refused}");
    assert!(refused.contains("did not admit"), "{refused}");
}

#[trace("FR-038-AC-145")]
#[test]
fn tc_226_qspec_recursive_records_admit() {
    let path = fixtures_dir().join(RECURSIVE);
    let package = read_json(&path).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(reading_of(&package).expect("fixture reading"), "admitted");
}

/// Apply a QSpec derived-shape mutation to every matching node in its positive package,
/// preserving the node key while refreshing both identity mirrors and the
/// package id. The fixture and mutation are always read from the QSpec checkout.
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
            let package_id = derived_with(&package, None)?;
            replace_at(&mut package, "/package_id/digest", &json!(package_id))?;
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

/// The base package and the selection vectors of the named checkout.
fn selection_inputs() -> (Value, Value) {
    let dir = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&dir).join(SELECTION_VECTORS))
        .unwrap_or_else(|why| panic!("{why}"));
    let base_path = base_path_of(&vectors).unwrap_or_else(|why| panic!("{why}"));
    let base =
        read_json(&proposal_dir_of(&dir).join(base_path)).unwrap_or_else(|why| panic!("{why}"));
    (base, vectors)
}

#[trace("TC-048", "FR-038-AC-113")]
#[test]
fn tc_048_qspec_version_free_selections_derive_the_recorded_package_id() {
    let (base, vectors) = selection_inputs();
    assert_no_problems(&selection_problems(&base, &vectors));
}

#[trace("TC-048", "FR-038-AC-113")]
#[test]
fn tc_048_qspec_selection_run_fails_closed() {
    assert!(fixtures_dir_of(None).is_err());
    assert!(fixtures_dir_of(Some("")).is_err());
    assert!(fixtures_dir_of(Some("/nonexistent/quire-specification")).is_err());
    let dir = fixtures_dir();
    assert!(read_json(&proposal_dir_of(&dir).join("no-such-vectors.json")).is_err());

    let (base, vectors) = selection_inputs();
    // Each case fails with its own problem, the rest of the vectors being sound.
    let problems_of = |vectors: &Value| selection_problems(&base, vectors);
    let mut wrong = vectors.clone();
    wrong["package_id"] = json!("0".repeat(64));
    assert!(mentions(&problems_of(&wrong), "not the recorded"));
    let mut empty = vectors.clone();
    empty["dependency_selections"] = json!([]);
    assert_eq!(
        problems_of(&empty),
        [format!(
            "{SELECTION_VECTORS} has no non-empty dependency_selections"
        )]
    );
    let mut absent = vectors.clone();
    absent
        .as_object_mut()
        .expect("vectors")
        .remove("dependency_selections");
    assert_eq!(
        problems_of(&absent),
        [format!(
            "{SELECTION_VECTORS} has no non-empty dependency_selections"
        )]
    );
    let mut unrecorded = vectors.clone();
    unrecorded
        .as_object_mut()
        .expect("vectors")
        .remove("package_id");
    assert_eq!(
        problems_of(&unrecorded),
        [format!("{SELECTION_VECTORS} has no string package_id")]
    );
    // The vectors name their base, and a base whose own package_id the
    // unchanged preimage does not derive fails on that.
    let mut unnamed = vectors.clone();
    unnamed.as_object_mut().expect("vectors").remove("base");
    assert!(base_path_of(&unnamed).is_err());
    let mut other_base = base.clone();
    other_base["package_id"]["digest"] = json!("1".repeat(64));
    assert!(mentions(
        &selection_problems(&other_base, &vectors),
        "not its own recorded package_id"
    ));
    // An entry carrying a `version` member is never the input of an identity.
    let mut versioned = vectors.clone();
    versioned["dependency_selections"][0]["version"] = json!("1.0.0");
    assert!(mentions(&problems_of(&versioned), "not a V2"));
    // Changing one entry's package id changes the derived identity, so a
    // recorded id that ignored the entries would fail here.
    let mut moved = vectors.clone();
    moved["dependency_selections"][0]["package_id"]["digest"] = json!("9".repeat(64));
    assert!(mentions(&problems_of(&moved), "not the recorded"));
}
