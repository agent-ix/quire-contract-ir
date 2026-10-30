// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-274 part A: every identity digest of the V2 reader is computed through
//! `quire-canonical` and is the digest it was before the move (FR-038 "Every
//! identity digest is computed through quire-canonical", FR-038-AC-89 through
//! FR-038-AC-92). The model document and lowering clauses (FR-038-AC-93
//! through FR-038-AC-95) are in `checked_package_v2_model_members.rs` and in
//! the unit tests of `lower.rs`.

use crate::support::checked_package::{
    canonical, evidence_for, nominal_fixture_members, positive_operation_identities, sha256_hex,
    v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_canonical::{Error, LimitKind};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CompleteLoweringProfileV2, CompleteLoweringRecordV2, NominalIdentityPreimage,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const RECORDED_NOMINAL: &str = "\
c9162d5e663855b38559e548f79621bcfb51eb08aa0b84eed6d333355c5456ac 79d77f491e2f327261dbc9f093a9c9737c475f4464af19b034bd948698b3c494
b4eec221df5de2eca7b131b2421ea954eeb6de07e17b4179ccddd413c5b958a2 48eb15ddc305107f8f13ce650c30425a994b11d0ec109bbf6541845997bb614d
fabe970245f926296d0e7499c65399574eeedbac1ab0c6f90caeb55c5b8a0242 17f32a9b746bf726bbe9f35405b18180844382dcd68e5fa157e9ce46ec97195a
5a263446f0e84319aab70efc5f27bbe04a0dbe0d037a5c201c660d1deddc27b3 e289ab36fcdddfa12208f3184810febb34bc042bea473b5d0cee55f02478d9cd
";

fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected V2 admission, got {other:?}"),
    }
}

/// Independent FR-092 key oracle over the published structural preimage.
/// The fixture builder does not supply this expected digest.
fn structural_oracle_key(wire: &Value) -> String {
    let key = &wire["node_id"];
    let semantic_type = (&wire["semantic_type"] != key).then(|| wire["semantic_type"].clone());
    let mut preimage = json!({
        "version": "quire.structural-node/v1",
        "node_tag": wire["node_tag"],
        "semantic_form": wire["semantic_form"],
        "semantic_type": semantic_type,
        "declaration": wire.get("declaration"),
        "recursion": null,
        "body": wire["body"],
    });
    if let Some(owner) = wire.get("owner") {
        preimage["owner"] = owner.clone();
    }
    sha256_hex(&canonical(&preimage))
}

fn application_oracle_key(wire: &Value) -> String {
    let mut preimage = json!({
        "version": "quire.application-node/v1",
        "node_tag": wire["node_tag"],
        "semantic_form": wire["semantic_form"],
        "semantic_type": wire["semantic_type"],
        "declaration": wire.get("declaration"),
        "recursion": null,
        "body": wire["body"],
    });
    if let Some(owner_model) = wire.get("owner_model") {
        preimage["owner_model"] = owner_model.clone();
    }
    sha256_hex(&canonical(&preimage))
}

/// Independent canonical-byte oracle for a lowered node's identity.
fn lowered_oracle_id(
    wire: &Value,
    dependencies: &[Value],
    bounds: &[Value],
    claims: &[Value],
) -> String {
    let mut projection = wire.clone();
    projection
        .as_object_mut()
        .expect("wire node")
        .remove("occurrences");
    sha256_hex(&canonical(&json!({
        "version": "quire.contract-ir.lowered-node/v1",
        "node": projection,
        "dependencies": dependencies,
        "bounds": bounds,
        "claims": claims,
    })))
}

/// Reference targets in the fixture's closed term grammar. Frame and
/// abstraction-relation bodies contribute their edges through `dependencies`.
fn body_targets(body: &Value) -> Vec<String> {
    let mut pending = vec![body];
    let mut targets = Vec::new();
    while let Some(term) = pending.pop() {
        let reference = match term["term"].as_str().expect("term tag") {
            "literal" => Some("type"),
            "reference" => Some("target"),
            "application" => {
                pending.extend(term["arguments"].as_array().expect("arguments"));
                Some("result_type")
            }
            "aggregate" => {
                pending.extend(term["members"].as_array().expect("members"));
                None
            }
            "binding" => {
                pending.push(&term["value"]);
                None
            }
            "dependency_reference" | "frame" | "abstraction_relation" => None,
            other => panic!("unexpected fixture body term {other}"),
        };
        if let Some(member) = reference {
            targets.push(
                term[member]["digest"]
                    .as_str()
                    .expect("body reference digest")
                    .to_owned(),
            );
        }
    }
    targets
}

/// Reachability oracle over `semantic_type`, declared dependencies and body
/// references, independently of the lowering result's returned key sets.
fn closure_oracle(
    wire: &Value,
    nodes: &BTreeMap<String, &Value>,
) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let request = wire["node_id"]["digest"].as_str().expect("node digest");
    let mut pending = vec![request.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(key) = pending.pop() {
        if !seen.insert(key.clone()) {
            continue;
        }
        let node = nodes.get(&key).expect("closed fixture graph");
        pending.push(
            node["semantic_type"]["digest"]
                .as_str()
                .expect("type digest")
                .to_owned(),
        );
        pending.extend(
            node["dependencies"]
                .as_array()
                .expect("dependencies")
                .iter()
                .map(|edge| {
                    edge["digest"]
                        .as_str()
                        .expect("dependency digest")
                        .to_owned()
                }),
        );
        pending.extend(body_targets(&node["body"]));
    }
    seen.remove(request);
    let ids = |tag: Option<&str>| {
        seen.iter()
            .filter(|key| tag.is_none_or(|tag| nodes[*key]["node_tag"] == tag))
            .map(|key| json!({"domain": "quire.checked-semantic-node/v1", "digest": key}))
            .collect::<Vec<_>>()
    };
    (ids(None), ids(Some("bounded_domain")), ids(Some("claim")))
}

type IdentityFixture = (&'static str, Value, Option<&'static str>);

/// ACs: FR-038-AC-89
#[trace("TC-048", "FR-038-AC-89")]
#[test]
fn tc_048_every_node_key_and_package_id_still_recomputes_and_lowering_is_unchanged() {
    let fixtures: [IdentityFixture; 3] = [
        ("v2_all_families", v2_all_families(), None),
        ("v2_nominal", v2_nominal(), Some(RECORDED_NOMINAL)),
        (
            "positive_operation_identities",
            positive_operation_identities(),
            None,
        ),
    ];
    for (name, value, recorded) in fixtures {
        let package = admit(&value);
        let requested: Vec<CheckedNodeId> = package
            .graph()
            .nodes
            .iter()
            .map(|node| node.node_id.clone())
            .collect();
        let profile = CompleteLoweringProfileV2 {
            reference: quire_contract_model::LoweringProfileRef {
                authority: "quire".into(),
                identity: "complete-v1".into(),
            },
            required_source_selections: Vec::new(),
            supported_tags: package.node_kinds().iter().map(|kind| kind.tag()).collect(),
            require_bounds: false,
            work_limit: u64::MAX,
        };
        let result = package.lower(&requested, &profile);
        let lowered: Vec<(String, String)> = requested
            .iter()
            .zip(&result.records)
            .map(|(key, record)| match record {
                CompleteLoweringRecordV2::Lowered { node } => {
                    (key.digest.to_string(), node.ir_id.digest.to_string())
                }
                other => panic!("{name}: expected a lowered record, got {other:?}"),
            })
            .collect();
        if let Some(recorded) = recorded {
            let expected: Vec<(String, String)> = recorded
                .lines()
                .map(|line| {
                    let (key, ir_id) = line.split_once(' ').expect("pair");
                    (key.to_owned(), ir_id.to_owned())
                })
                .collect();
            assert_eq!(lowered, expected, "{name}: recorded identities");
        } else {
            let wires = value["semantic_graph"]["nodes"].as_array().expect("nodes");
            assert_eq!(lowered.len(), wires.len(), "{name}: node count");
            let by_key: BTreeMap<String, &Value> = wires
                .iter()
                .map(|wire| {
                    (
                        wire["node_id"]["digest"]
                            .as_str()
                            .expect("node digest")
                            .to_owned(),
                        wire,
                    )
                })
                .collect();
            assert_eq!(by_key.len(), wires.len(), "{name}: unique keys");
            for ((actual, wire), record) in lowered.iter().zip(wires).zip(&result.records) {
                let derived = if wire["body"]["term"] == "application" {
                    application_oracle_key(wire)
                } else {
                    structural_oracle_key(wire)
                };
                assert_eq!(actual.0, derived, "{name}: canonical node key");
                let (dependencies, bounds, claims) = closure_oracle(wire, &by_key);
                let CompleteLoweringRecordV2::Lowered { node } = record else {
                    panic!("{name}: expected a lowered record");
                };
                assert_eq!(
                    serde_json::to_value(&node.dependencies).expect("dependencies"),
                    json!(dependencies)
                );
                assert_eq!(
                    serde_json::to_value(&node.bounds).expect("bounds"),
                    json!(bounds)
                );
                assert_eq!(
                    serde_json::to_value(&node.claims).expect("claims"),
                    json!(claims)
                );
                assert_eq!(
                    actual.1,
                    lowered_oracle_id(wire, &dependencies, &bounds, &claims),
                    "{name}: canonical lowered identity"
                );
            }
        }
        let id = result
            .package
            .package_id()
            .expect("the package is identified");
        let bytes = result.package.canonical_bytes().expect("package bytes");
        let encoded: Value = serde_json::from_slice(bytes).expect("package JSON");
        assert_eq!(
            canonical(&encoded),
            bytes,
            "{name}: canonical package bytes"
        );
        let preimage = json!({
            "contract_version": encoded["contract_version"],
            "package": encoded["package"],
            "semantic_graph": encoded["semantic_graph"],
            "feature_accounting": encoded["feature_accounting"],
        });
        assert_eq!(
            sha256_hex(&canonical(&preimage)),
            id.digest.as_ref(),
            "{name}"
        );
    }
}

/// The preimage JSON text of each nominal version, written out by hand with
/// members in RFC 8785 order, and the node key the fixture recorded for it
/// (computed by the fixture generator, not by the code under test).
const NOMINAL_TEXT: [(&str, &str); 4] = [
    (
        r#"{"case":"ACTIVE","declaration_node_id":{"digest":"b4eec221df5de2eca7b131b2421ea954eeb6de07e17b4179ccddd413c5b958a2","domain":"quire.checked-semantic-node/v1"},"version":"quire.enum-member-node/v1"}"#,
        "c9162d5e663855b38559e548f79621bcfb51eb08aa0b84eed6d333355c5456ac",
    ),
    (
        r#"{"members":["ACTIVE","DONE"],"ordered":false,"owner":{"authority":"agent-ix","identity":"quire.fixture.source/v1","kind":"source"},"qualified_declaration":["Example","Status"],"version":"quire.enum-declaration-node/v1"}"#,
        "b4eec221df5de2eca7b131b2421ea954eeb6de07e17b4179ccddd413c5b958a2",
    ),
    (
        r#"{"dimension_node_id":{"digest":"5a263446f0e84319aab70efc5f27bbe04a0dbe0d037a5c201c660d1deddc27b3","domain":"quire.checked-semantic-node/v1"},"offset":{"denominator":"1","numerator":"0"},"owner":{"authority":"agent-ix","identity":"quire.fixture.source/v1","kind":"source"},"qualified_declaration":["Example","Metre"],"scale":{"denominator":"1","numerator":"1"},"target_unit_node_id":null,"version":"quire.unit-node/v1"}"#,
        "fabe970245f926296d0e7499c65399574eeedbac1ab0c6f90caeb55c5b8a0242",
    ),
    (
        r#"{"owner":{"authority":"agent-ix","identity":"quire.fixture.definition.selection/v1","kind":"definition"},"qualified_declaration":["Example","Length"],"terms":[],"version":"quire.dimension-node/v1"}"#,
        "5a263446f0e84319aab70efc5f27bbe04a0dbe0d037a5c201c660d1deddc27b3",
    ),
];

/// ACs: FR-038-AC-89, FR-038-AC-90
#[trace("TC-048", "FR-038-AC-89", "FR-038-AC-90")]
#[test]
fn tc_048_a_nominal_digest_takes_a_byte_limit_and_hashes_the_expected_bytes() {
    // The four preimages are the fixture's, in the same order.
    let members = nominal_fixture_members();
    assert_eq!(members.len(), NOMINAL_TEXT.len());
    for ((preimage, key), (text, expected_key)) in members.iter().zip(NOMINAL_TEXT) {
        assert_eq!(key, expected_key, "the fixture's recorded key");
        // Expected canonical bytes written out above, not produced by a call
        // into the code under test; the recorded key is the second oracle.
        assert_eq!(sha256_hex(text.as_bytes()), expected_key);
        let typed: NominalIdentityPreimage =
            serde_json::from_value(preimage.clone()).expect("typed preimage");
        let bytes = quire_canonical::to_vec(&typed, quire_canonical::Limits::new(1 << 20))
            .expect("encodes");
        assert_eq!(String::from_utf8(bytes).expect("utf-8"), text);
        let length = u64::try_from(text.len()).expect("length");
        assert_eq!(typed.digest(length).as_deref().ok(), Some(expected_key));
        match typed.digest(length - 1) {
            Err(Error::Limit(limit)) => {
                assert_eq!(limit.kind, LimitKind::CanonicalBytes);
                assert_eq!(limit.bound, length - 1);
            }
            other => panic!("expected the byte-limit refusal, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Source scans.
// ---------------------------------------------------------------------------

pub(crate) fn repository_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn rust_files_under(directory: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(directory)
        .expect("directory reads")
        .map(|entry| entry.expect("entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files_under(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// `text` without its `#[cfg(test)]` items, wherever they are in the file: an
/// attributed item is skipped from the attribute to the closing brace at the
/// attribute's own indentation, or to its first line ending in `;`. The code
/// after a test item is kept.
pub(crate) fn production_source(text: &str) -> String {
    let mut kept = String::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim_start() != "#[cfg(test)]" {
            kept.push_str(line);
            kept.push('\n');
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let closing = format!("{}}}", " ".repeat(indent));
        // The attributed item's first line decides: an item that ends there
        // (`mod tests;`, a `use`) is only that line, and any other runs to its
        // closing brace.
        let mut first = true;
        for skipped in lines.by_ref() {
            let item = skipped.trim_start();
            if item.starts_with("#[") || item.starts_with("//") {
                continue;
            }
            if first && item.ends_with(';') {
                break;
            }
            first = false;
            if skipped.starts_with(&closing) {
                break;
            }
        }
    }
    kept
}

/// Out-of-line module names under this repository's one-line default module
/// layout. Inline modules stay with their parent. Unrecognized layouts cannot
/// exclude a file: the physical source inventory is always scanned by default.
fn out_of_line_modules(text: &str) -> std::collections::BTreeSet<&str> {
    // This scan does not interpret path overrides. Keep every physical file
    // when a module's layout is not the actual default layout handled below.
    if text
        .lines()
        .any(|line| line.trim_start().starts_with("#[path"))
    {
        return std::collections::BTreeSet::new();
    }
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (visibility, declaration) = line.split_once("mod ")?;
            if !visibility.is_empty() && !visibility.trim().starts_with("pub") {
                return None;
            }
            let name = declaration.strip_suffix(';')?.trim();
            (!name.is_empty() && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'))
                .then_some(name)
        })
        .collect()
}

/// Resolve only actual default-layout module files from this source inventory.
fn module_file(
    parent: &Path,
    name: &str,
    sources: &std::collections::BTreeMap<PathBuf, String>,
) -> Option<PathBuf> {
    let directory = if parent.file_name().is_some_and(|file| file == "mod.rs") {
        parent.parent()?.to_path_buf()
    } else {
        parent.parent()?.join(parent.file_stem()?)
    };
    let file = directory.join(name).with_extension("rs");
    let directory_file = directory.join(name).join("mod.rs");
    match (
        sources.contains_key(&file),
        sources.contains_key(&directory_file),
    ) {
        (true, false) => Some(file),
        (false, true) => Some(directory_file),
        // Missing, ambiguous or nondefault layouts receive no exclusion.
        _ => None,
    }
}

/// Derive exclusions from actual cfg(test) module ancestry, never filenames.
/// A child's test-only status is inherited only through an actual declaration.
/// This is a text scan, not a Rust parser: comments, raw strings and inline
/// module nesting can resemble declarations. The caller pins the exact excluded
/// set so an unexpected exclusion fails the audit instead of hiding production.
fn test_only_module_files(
    sources: &std::collections::BTreeMap<PathBuf, String>,
) -> std::collections::BTreeSet<PathBuf> {
    let mut excluded = std::collections::BTreeSet::new();
    let mut pending = Vec::new();
    for (path, raw) in sources {
        let production = production_source(raw);
        let all = out_of_line_modules(raw);
        let live = out_of_line_modules(&production);
        for name in all.difference(&live) {
            if let Some(child) = module_file(path, name, sources) {
                pending.push(child);
            }
        }
    }
    while let Some(path) = pending.pop() {
        if !excluded.insert(path.clone()) {
            continue;
        }
        if let Some(raw) = sources.get(&path) {
            for name in out_of_line_modules(raw) {
                if let Some(child) = module_file(&path, name, sources) {
                    pending.push(child);
                }
            }
        }
    }
    excluded
}

/// Every physical source file under checked_package, except actual cfg(test)
/// modules and their declared descendants, with inline cfg(test) items removed.
fn checked_package_sources() -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    rust_files_under(
        &repository_path("crates/quire-contract-model/src/checked_package"),
        &mut files,
    );
    let sources: std::collections::BTreeMap<_, _> = files
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).expect("source reads");
            (path, text)
        })
        .collect();
    let test_only = test_only_module_files(&sources);
    // This is a regression oracle, not an exclusion allow-list: ancestry above
    // computes the set, and any added or missing exclusion must fail this audit.
    let expected: std::collections::BTreeSet<_> = [
        "lower/ceiling_tests.rs",
        "model_fields/tests.rs",
        "model_members/tests.rs",
        "model_members/tests/intake_retention.rs",
    ]
    .map(|relative| {
        repository_path("crates/quire-contract-model/src/checked_package/v2").join(relative)
    })
    .into_iter()
    .collect();
    assert_eq!(test_only, expected, "exact cfg(test) source ancestry");
    sources
        .into_iter()
        .filter(|(path, _)| !test_only.contains(path))
        .map(|(path, text)| (path, production_source(&text)))
        .collect()
}

/// ACs: FR-038-AC-91
#[trace("TC-048", "FR-038-AC-91")]
#[test]
fn tc_048_the_checked_package_source_holds_no_encoder_of_its_own() {
    let sources = checked_package_sources();
    assert!(
        sources.len() > 10,
        "the scan reads the checked-package files"
    );
    // Actual nested unit fixtures inherit the cfg(test) of model_members' module.
    // The production origin decoder remains in the scan.
    assert!(!sources
        .iter()
        .any(|(path, _)| path.ends_with("model_members/tests/intake_retention.rs")));
    assert!(sources
        .iter()
        .any(|(path, _)| path.ends_with("model_members/intake_origin.rs")));
    // Negative control: a production module named tests is still production,
    // including a nested child and the same serializer forbidden below.
    let root = PathBuf::from("synthetic/mod.rs");
    let production = PathBuf::from("synthetic/tests.rs");
    let nested = PathBuf::from("synthetic/tests/nested.rs");
    let control = std::collections::BTreeMap::from([
        (root, "mod tests;\n".to_owned()),
        (production.clone(), "mod nested;\n".to_owned()),
        (
            nested.clone(),
            "fn encode() { serde_json::to_vec(&value); }\n".to_owned(),
        ),
    ]);
    let test_only = test_only_module_files(&control);
    assert!(!test_only.contains(&production));
    assert!(!test_only.contains(&nested));
    assert_eq!(
        production_source(&control[&nested])
            .matches("serde_json::to_vec")
            .count(),
        1
    );
    for symbol in [
        "CanonicalWriter",
        "canonical_envelope_bytes",
        "digest_json",
        "serde_json_canonicalizer",
        "value_to_vec",
        "serde_json::to_vec",
        "to_vec_pretty",
        "to_writer",
    ] {
        for (path, text) in &sources {
            let hits = text.matches(symbol).count();
            assert_eq!(hits, 0, "{}: `{symbol}` x{hits}", path.display());
        }
    }
    // `serde_json::to_value` is limited to the reader's first-difference
    // pointer comparisons, the enum-member literal-type comparison, and one
    // typed producer draft projection into the existing pre-read candidate
    // algorithm. That projection is not used by admitted package identity or
    // lossless intake; the candidate still uses quire-canonical for its digest.
    // Any other call, or another in those files, fails.
    let expected: [(&str, usize); 3] =
        [("v2/mod.rs", 3), ("v2/intake.rs", 0), ("v2/identity.rs", 1)];
    for (path, text) in &sources {
        let hits = text.matches("serde_json::to_value").count();
        let allowed = expected
            .iter()
            .find(|(suffix, _)| path.ends_with(suffix))
            .map_or(0, |(_, count)| *count);
        assert_eq!(hits, allowed, "{}: `serde_json::to_value`", path.display());
    }
    for (path, text) in &sources {
        // No `impl Encode for serde_json::Value` of this repository's own.
        assert!(
            !text.contains("Encode for Value") && !text.contains("Encode for serde_json::Value"),
            "{}",
            path.display()
        );
        // No hand-rolled drop of a `Value`: it goes through `drop_value`.
        assert!(!text.contains("fn dismantle"), "{}", path.display());
    }
    let manifest = fs::read_to_string(repository_path("crates/quire-contract-model/Cargo.toml"))
        .expect("manifest reads");
    let line = manifest
        .lines()
        .find(|line| line.starts_with("quire-canonical"))
        .expect("quire-canonical dependency");
    assert!(line.contains("features = [\"serde_json\"]"), "{line}");
}

/// ACs: FR-038-AC-90
#[trace("TC-048", "FR-038-AC-90")]
#[test]
fn tc_048_no_encode_in_the_checked_package_source_passes_u64_max_or_a_literal_cap() {
    for (path, text) in checked_package_sources() {
        for (position, _) in text.match_indices("Limits::new(") {
            let argument = &text[position + "Limits::new(".len()..];
            let argument = argument.split(')').next().unwrap_or("");
            assert!(
                !argument.contains("u64::MAX")
                    && !argument.chars().next().is_some_and(|c| c.is_ascii_digit()),
                "{}: Limits::new({argument})",
                path.display()
            );
        }
        for (position, _) in text.match_indices(".digest(") {
            let argument = &text[position + ".digest(".len()..];
            let argument = argument.split(')').next().unwrap_or("");
            assert!(
                !argument.contains("u64::MAX")
                    && !argument.chars().next().is_some_and(|c| c.is_ascii_digit()),
                "{}: .digest({argument})",
                path.display()
            );
        }
    }
    // The reader passes its configured byte limit to the nominal digest.
    let reader = fs::read_to_string(repository_path(
        "crates/quire-contract-model/src/checked_package/v2/mod.rs",
    ))
    .expect("source reads");
    assert!(reader.contains("limits.bytes"));
    let identity = fs::read_to_string(repository_path(
        "crates/quire-contract-model/src/checked_package/v2/identity.rs",
    ))
    .expect("source reads");
    assert!(identity.contains(".digest(bytes)"));
}

/// ACs: FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_the_lowering_makes_no_expect_or_unwrap_on_an_encode() {
    for path in [
        "crates/quire-contract-model/src/checked_package/v2/lower.rs",
        "crates/quire-contract-model/src/checked_package/v2/contract_package.rs",
    ] {
        let text = fs::read_to_string(repository_path(path)).expect("source reads");
        let production = production_source(&text);
        for call in [".expect(", ".unwrap(", ".unwrap_or_default(", "panic!("] {
            assert_eq!(production.matches(call).count(), 0, "`{call}` in {path}");
        }
    }
}

/// ACs: FR-038-AC-92
#[trace("TC-048", "FR-038-AC-92")]
#[test]
fn tc_048_preimages_take_the_path_their_depth_allows() {
    let sources = checked_package_sources();
    let all: String = sources.iter().map(|(_, text)| text.as_str()).collect();
    for encoded in [
        "impl Encode for ApplicationNodePreimage",
        "impl Encode for LoweredNodePreimage",
        "impl Encode for ContractPackageWireV1",
    ] {
        assert!(all.contains(encoded), "{encoded}");
    }
    let derives_fixed_shape = |name: &str| {
        let marker = format!("struct {name}");
        let enum_marker = format!("enum {name}");
        let lines: Vec<&str> = all.lines().collect();
        lines.iter().enumerate().any(|(position, line)| {
            let declares = line.contains(&marker) || line.contains(&enum_marker);
            declares
                && lines[..position]
                    .iter()
                    .rev()
                    .take_while(|above| {
                        let above = above.trim_start();
                        above.starts_with("#[") || above.starts_with("///")
                    })
                    .any(|above| {
                        above.trim_start().starts_with("#[derive(") && above.contains("FixedShape")
                    })
        })
    };
    assert!(
        derives_fixed_shape("NominalIdentityPreimage"),
        "nominal identity derives FixedShape"
    );
    assert!(
        all.contains("impl<B: Encode> Encode for StructuralPreimage"),
        "group bodies require a fallible streaming structural preimage"
    );
    for encoded in [
        "ApplicationNodePreimage",
        "LoweredNodePreimage",
        "ContractPackageWireV1",
    ] {
        assert!(
            !derives_fixed_shape(encoded),
            "{encoded} does not derive FixedShape"
        );
    }
    assert!(
        !all.contains("impl FixedShape for"),
        "no hand-written FixedShape"
    );
    assert!(!all.contains("impl quire_canonical::FixedShape for"));
}
