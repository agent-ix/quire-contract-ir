// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-107, AC-112 and AC-113: QSpec's CheckedPackage V2 fixtures and
//! vectors against the production reader.
//!
//! AC-107: the positive fixtures admit end to end. AC-112: every mutation of
//! `adverse.json` refuses as recorded, with no identity refreshed, bar the
//! listed expected failures, and each body-grammar `flattened` form is not
//! refused `malformed_wire`. AC-113: the version-free dependency selections of
//! `dependency-selection-vectors.json` derive the recorded `package_id`
//! through the reader's own derivation.
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
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageIdentityPreimageV2, CheckedPackageLockV2,
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

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

/// What the production reader makes of `package`: `admitted`, `incomplete`, or
/// `refused:<code>` with `/<cause>` where the reader gives one.
fn reading_of(package: &Value) -> Result<String, String> {
    // The reader takes canonical bytes; the published files are indented.
    let bytes = serde_json::to_vec(package).map_err(|error| error.to_string())?;
    let mut evidence = CheckedPackageEvidence::new();
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
    Ok(
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => "admitted".to_owned(),
            CheckedPackageV2ReadResult::Incomplete(_) => "incomplete".to_owned(),
            CheckedPackageV2ReadResult::Refused(refusal) => match refusal.cause {
                Some(cause) => format!("refused:{}/{}", code_word(refusal.code), cause_word(cause)),
                None => format!("refused:{}", code_word(refusal.code)),
            },
        },
    )
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
        Code::AmbiguousDeclaration => "ambiguous_declaration",
        Code::MissingImport => "missing_import",
        Code::UnknownProfile => "unknown_profile",
    }
}

/// The wire word of a refusal cause, with no wildcard for the reason above.
fn cause_word(cause: Cause) -> &'static str {
    match cause {
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
        Cause::UnsupportedSelection => "unsupported-selection",
        Cause::WrongSelectionRole => "wrong-selection-role",
        Cause::InexactInteger => "inexact-integer",
        Cause::InexactNumber => "inexact-number",
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

/// FR-038-AC-112's expected-failure list: the body-grammar mutations the reader
/// accepts as wire today and refuses at a later identity check. IR-495 makes the
/// reader enforce the flat v2 wire (nested inline terms refused `malformed_wire`),
/// and deletes these entries.
const EXPECTED_FAILURES: &[ExpectedFailure] = &[
    ExpectedFailure {
        id: "application-in-application-arguments",
        refusal: "refused:invalid_package/stale-node-key",
        ticket: "IR-495",
    },
    ExpectedFailure {
        id: "application-in-aggregate-members",
        refusal: "refused:invalid_semantic_graph",
        ticket: "IR-495",
    },
    ExpectedFailure {
        id: "application-in-binding-value",
        refusal: "refused:invalid_semantic_graph",
        ticket: "IR-495",
    },
    ExpectedFailure {
        id: "aggregate-in-group-members",
        refusal: "refused:stale_dependency",
        ticket: "IR-495",
    },
    ExpectedFailure {
        id: "binding-as-body-root",
        refusal: "refused:stale_dependency",
        ticket: "IR-495",
    },
];

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
                None if given != recorded => {
                    return Err(format!("read as {given}, not the recorded {recorded}"));
                }
                Some(listed) if given == recorded => {
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
    quire_canonical::sha256(&preimage, Limits::new(DERIVATION_CEILING))
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
    match derived_with(base, None) {
        Ok(unchanged) if unchanged == recorded => {
            problems.push("the unchanged base derives the recorded package_id too".into());
        }
        Ok(_) => {}
        Err(why) => problems.push(why),
    }
    problems
}

/// Tracing: TC-048, FR-038-AC-107
#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn tc_048_qspec_positive_fixtures_admit() {
    let dir = fixtures_dir();
    for name in POSITIVE {
        let path = dir.join(name);
        let text = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("{name}: {} cannot be read: {error}", path.display()));
        let package: Value = serde_json::from_slice(&text)
            .unwrap_or_else(|error| panic!("{name}: not JSON: {error}"));
        // The reader takes canonical bytes; the published files are indented.
        let bytes = serde_json::to_vec(&package).expect("a JSON value serializes");
        let mut evidence = CheckedPackageEvidence::new();
        for feature in package["lock"]["required_features"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: lock.required_features is not an array"))
        {
            evidence.support_feature(
                feature
                    .as_str()
                    .unwrap_or_else(|| panic!("{name}: a required feature is not a string")),
            );
        }
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(admitted) => {
                let derived =
                    serde_json::to_value(admitted.package_id()).expect("a package id serializes");
                assert_eq!(
                    derived, package["package_id"],
                    "{name}: recorded package_id"
                );
            }
            other => panic!("{name} does not admit: {other:?}"),
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

/// Tracing: TC-048, FR-038-AC-112
#[trace("TC-048", "FR-038-AC-112")]
#[test]
fn tc_048_qspec_adverse_mutations_refuse_as_recorded() {
    let (base, adverse) = adverse_inputs();
    assert_no_problems(&adverse_problems(&base, &adverse, EXPECTED_FAILURES));
    // A listed id is a mutation of the file, and each entry names its owner.
    for listed in EXPECTED_FAILURES {
        assert!(listed.ticket.starts_with("IR-"), "{}: owner", listed.id);
    }
}

/// Tracing: TC-048, FR-038-AC-112
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
    let failing = |adverse: &Value, expected: &[ExpectedFailure]| {
        !adverse_problems(&base, adverse, expected).is_empty()
    };

    // A list absent or empty fails.
    assert!(failing(
        &json!({"body_grammar_mutations": [body.clone()]}),
        &[]
    ));
    assert!(failing(
        &json!({"structural_mutations": [], "body_grammar_mutations": [body.clone()]}),
        &[]
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
    let base = read_json(&dir.join(MUTATED_BASE)).unwrap_or_else(|why| panic!("{why}"));
    (base, vectors)
}

/// Tracing: TC-048, FR-038-AC-113
#[trace("TC-048", "FR-038-AC-113")]
#[test]
fn tc_048_qspec_version_free_selections_derive_the_recorded_package_id() {
    let (base, vectors) = selection_inputs();
    assert_no_problems(&selection_problems(&base, &vectors));
}

/// Tracing: TC-048, FR-038-AC-113
#[trace("TC-048", "FR-038-AC-113")]
#[test]
fn tc_048_qspec_selection_run_fails_closed() {
    assert!(fixtures_dir_of(None).is_err());
    assert!(fixtures_dir_of(Some("")).is_err());
    assert!(fixtures_dir_of(Some("/nonexistent/quire-specification")).is_err());
    let dir = fixtures_dir();
    assert!(read_json(&proposal_dir_of(&dir).join("no-such-vectors.json")).is_err());

    let (base, vectors) = selection_inputs();
    let failing = |vectors: &Value| !selection_problems(&base, vectors).is_empty();
    // A differing recorded identity, an absent or empty list and an absent id fail.
    let mut wrong = vectors.clone();
    wrong["package_id"] = json!("0".repeat(64));
    assert!(failing(&wrong));
    let mut empty = vectors.clone();
    empty["dependency_selections"] = json!([]);
    assert!(failing(&empty));
    let mut absent = vectors.clone();
    absent
        .as_object_mut()
        .expect("vectors")
        .remove("dependency_selections");
    assert!(failing(&absent));
    let mut unrecorded = vectors.clone();
    unrecorded
        .as_object_mut()
        .expect("vectors")
        .remove("package_id");
    assert!(failing(&unrecorded));
    // An entry carrying a `version` member is never the input of an identity.
    let mut versioned = vectors.clone();
    versioned["dependency_selections"][0]["version"] = json!("1.0.0");
    let problems = selection_problems(&base, &versioned);
    assert!(
        problems.iter().any(|problem| problem.contains("not a V2")),
        "{problems:?}"
    );
    // The harness derives through the reader's own derivation, so an entry
    // whose package id changes moves the identity.
    let mut moved = vectors.clone();
    moved["dependency_selections"][0]["package_id"]["digest"] = json!("9".repeat(64));
    assert!(failing(&moved));
}
