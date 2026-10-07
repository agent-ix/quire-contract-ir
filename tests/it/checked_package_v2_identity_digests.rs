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
use quire_contract_ir::{
    CheckedNodeId, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CompleteContractNodeV2, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    NominalIdentityPreimage,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// `ir_id` of every lowered node, then the lowered package's `package_id` and
/// canonical byte length, recorded from the lowering before the move to
/// `quire-canonical`. One `node ir_id` pair per line, in graph order.
const RECORDED_ALL_FAMILIES: &str = "\
9964390677844ad66b781babdbfa95933bc2b16ef1e86f67005966b77e6db3aa 742a476c3eb3106e7a35ca7a056d806245cbaf50526cdd6749d61330ad7bf187
64122a85d03d18a2cc2159533da9a64dcf38cbd51d2dd1dc787684b2c6dca3ff f9ec7e88cabdf27428942bdac05604c925502435d8880c4342e74acfe270f8b2
da7206523158146e82400a142bf315ccbaab12be85cd9bb7306f610e857dce00 a4a410f63d9ffaf6485dced3497181ebf555d792dca9eaa2e5307e49be943e61
dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd 205694e20f84256d05f36b48ae4530211a6eaf550d2499141b88ed11c23d25ab
eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee ec8daec331d3141d61437f3f034afaf149d1ff0b6056e54bace784d589ee9507
46ba3240097c31e20c060e8d22ca6d9c8367157eeaef4d07df58ff4564b03f4d 93c499162238525068185c7640408b49ba0bdfd781b623f3bce43f380f620d6f
1010101010101010101010101010101010101010101010101010101010101010 f591e6be7787b9ae6c05ef734bf57f7988ae6bed27b18dd995e2e24a5b28d143
2020202020202020202020202020202020202020202020202020202020202020 adad6936f27bc9d4a3ec53825743dd73dc16f1ba999004c59a8a27d3e26ddafe
3030303030303030303030303030303030303030303030303030303030303030 154f979c1f76c16cc939cccc05a0b33a95eb2a1b8ca0bf9823d1cbd72152f1b3
604ab43bb381922319c084f90fb8f3a63d913263e64e11abfe6d930b7a485683 18d00b7f5f9b92f4ac4489ae1c19f98f8f118adc6d29951d805543d7e934d3e0
bb759058b3950264ddf00aa88788145c3fff9d50c1d978385d9d37ab937d34fd 5e3cc03945d41279fa91322c4bcb59aeaffd73fdf7c233ed7df070e80020093d
570ad4c0f445d4b9205ac8f700c7354b371470f07574c9d50aaf7fc2c52985b0 4411df292705a9f5f5fbba48a20ded700d16d7587b2a183695154f5c9a2d94ad
7070707070707070707070707070707070707070707070707070707070707070 aa1bc18362fe82ef94f0edcc49ad7b4c4a9c19809f0dc58e7a32abe114885784
8080808080808080808080808080808080808080808080808080808080808080 f80587b8ed259103feb9510015ed584ba1f2f8bb0c11a531e1854ce85adf591b
1515151515151515151515151515151515151515151515151515151515151515 5218c467de614e9074cd7c3edb1dbf4559c109a5dccf0637699e6e2afd2dc744
1616161616161616161616161616161616161616161616161616161616161616 1d1134d4d43f5f02e14ccf05e9ece8e91a8881eee64dd1685e32e2259717368b
f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0 e68f4ea524756a27c7a80079128dcf8b5b7dcb0dee34b5020b8ba266b1a64db5
a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1 7bc463ef2cd246231a48e0f812baed64d2f61d8e724ca466bc0b53b0e24f91b2
9ff55d90c0aee49063e5089611362385e897453178ad2c64163014c6e36afbee 48017a73a26ce8a906d3bc302f082ff4b5ff44e35415765709851625cfda267a
a677efccc10c1960b68811f6d9904c6fdae6c3a99bc62a1f1cea9c3653cc4b9a e59e72574f82dd7c7fb40786e0060df0c4177a393d38a2b04da2a8b8a21e661e
07f6dca966d22bde13d3bb198f12610e57d8e1e04d0476bbab03f405d2b04e32 aeef2fa1c3789c2ff17389d2b237ed1c2ba3cc9d26b8fda79c6c3975a9af2114
a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4 714815380b2d282c308ba5d34dbec2d469160a801dcc96486704cbddf29b1913
";
const ALL_FAMILIES_PACKAGE: (&str, usize) = (
    "ff928e306afa74914b5674cea6060c94458c7b12e8eb06a0d966c01b3059061f",
    38370,
);

const RECORDED_NOMINAL: &str = "\
c9162d5e663855b38559e548f79621bcfb51eb08aa0b84eed6d333355c5456ac 79d77f491e2f327261dbc9f093a9c9737c475f4464af19b034bd948698b3c494
b4eec221df5de2eca7b131b2421ea954eeb6de07e17b4179ccddd413c5b958a2 48eb15ddc305107f8f13ce650c30425a994b11d0ec109bbf6541845997bb614d
fabe970245f926296d0e7499c65399574eeedbac1ab0c6f90caeb55c5b8a0242 17f32a9b746bf726bbe9f35405b18180844382dcd68e5fa157e9ce46ec97195a
5a263446f0e84319aab70efc5f27bbe04a0dbe0d037a5c201c660d1deddc27b3 e289ab36fcdddfa12208f3184810febb34bc042bea473b5d0cee55f02478d9cd
";
const NOMINAL_PACKAGE: (&str, usize) = (
    "d3c658c9f22716bd2f602f13040e59725a29ceba66d508c91a78967f27463f6a",
    7113,
);

const RECORDED_OPERATIONS: &str = "\
9964390677844ad66b781babdbfa95933bc2b16ef1e86f67005966b77e6db3aa 742a476c3eb3106e7a35ca7a056d806245cbaf50526cdd6749d61330ad7bf187
cafecafecafecafecafecafecafecafecafecafecafecafecafecafecafecafe 88614f70de145e5eee7e55f61f4247d959ade92f6ee1a735d1beeab4506ab044
d00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00d 3691777c4507fcbc1c730c20af9ccc916c5e223086b5dabe4e75d25eca39da29
f00df00df00df00df00df00df00df00df00df00df00df00df00df00df00df00d 9b8fad0018827aaa5c6be8b7dd71d66f0c1303569ba1defeecd188d88f007847
37b50f2dd7623c54a5ca2b73b2fc6a965b2eeeccb6a8ccd47b8f137b32ff4f3f d84e850e60b9b483e01f64de7a8408f58ac57bfda342992d763e41a6dfc5465e
";
const OPERATIONS_PACKAGE: (&str, usize) = (
    "c10dac8e60c89fd64a3838f21bc482e560a84baef299808a307cb09604ddeaa4",
    7949,
);

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
    let preimage = json!({
        "version": "quire.structural-node/v1",
        "node_tag": wire["node_tag"],
        "semantic_form": wire["semantic_form"],
        "semantic_type": semantic_type,
        "declaration": wire.get("declaration"),
        "recursion": null,
        "body": wire["body"],
    });
    sha256_hex(&canonical(&preimage))
}

fn application_oracle_key(wire: &Value) -> String {
    sha256_hex(&canonical(&json!({
        "version": "quire.application-node/v1",
        "node_tag": wire["node_tag"],
        "semantic_form": wire["semantic_form"],
        "semantic_type": wire["semantic_type"],
        "declaration": wire.get("declaration"),
        "recursion": null,
        "body": wire["body"],
    })))
}

/// Independent canonical-byte oracle for a lowered node's identity.
fn lowered_oracle_id(wire: &Value, lowered: &CompleteContractNodeV2) -> String {
    let mut projection = wire.clone();
    projection
        .as_object_mut()
        .expect("wire node")
        .remove("occurrences");
    sha256_hex(&canonical(&json!({
        "version": "quire.contract-ir.lowered-node/v1",
        "node": projection,
        "dependencies": lowered.dependencies,
        "bounds": lowered.bounds,
        "claims": lowered.claims,
    })))
}

/// Tracing: TC-048
/// ACs: FR-038-AC-89
#[trace("TC-048", "FR-038-AC-89")]
#[test]
fn tc_048_every_node_key_and_package_id_still_recomputes_and_lowering_is_unchanged() {
    let fixtures: [(&str, Value, &str, (&str, usize)); 3] = [
        (
            "v2_all_families",
            v2_all_families(),
            RECORDED_ALL_FAMILIES,
            ALL_FAMILIES_PACKAGE,
        ),
        (
            "v2_nominal",
            v2_nominal(),
            RECORDED_NOMINAL,
            NOMINAL_PACKAGE,
        ),
        (
            "positive_operation_identities",
            positive_operation_identities(),
            RECORDED_OPERATIONS,
            OPERATIONS_PACKAGE,
        ),
    ];
    for (name, value, recorded, (package_id, length)) in fixtures {
        // Admission re-derives every nominal, application and structural node
        // key and the `package_id` the fixture recorded before the move.
        let package = admit(&value);
        let requested: Vec<CheckedNodeId> = package
            .graph()
            .nodes
            .iter()
            .map(|node| node.node_id.clone())
            .collect();
        let profile = CompleteLoweringProfileV2 {
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
        let expected: Vec<(String, String)> = recorded
            .lines()
            .map(|line| {
                let (key, ir_id) = line.split_once(' ').expect("pair");
                (key.to_owned(), ir_id.to_owned())
            })
            .collect();
        if name == "v2_nominal" {
            assert_eq!(lowered, expected, "{name}: `ir_id` values");
        } else {
            // Exact recorded pairs remain the oracle where their identity
            // preimage has not changed. IR-627's changed structural keys use
            // independent test-side canonical preimages; QSpec/QSL emitted
            // fixtures supply the authoritative golden separately.
            assert_eq!(lowered.len(), expected.len(), "{name}: node count");
            for (((actual, previous), wire), record) in lowered
                .iter()
                .zip(&expected)
                .zip(value["semantic_graph"]["nodes"].as_array().expect("nodes"))
                .zip(&result.records)
            {
                if wire.get("owner").is_some() {
                    continue;
                }
                if wire.get("nominal_identity_preimage").is_some()
                    || matches!(wire["node_tag"].as_str(), Some("model" | "relation"))
                    || (wire["node_tag"] == "correspondence"
                        && wire["semantic_form"] == "abstraction_relation")
                {
                    assert_eq!(actual, previous, "{name}: owner-free node");
                    continue;
                }
                let derived = if wire["body"]["term"] == "application" {
                    application_oracle_key(wire)
                } else {
                    structural_oracle_key(wire)
                };
                if derived == previous.0 {
                    assert_eq!(actual, previous, "{name}: unchanged structural node");
                } else {
                    let CompleteLoweringRecordV2::Lowered { node } = record else {
                        panic!("{name}: expected a lowered record");
                    };
                    assert_eq!(actual.0, derived, "{name}: changed node key");
                    assert_eq!(
                        actual.1,
                        lowered_oracle_id(wire, node),
                        "{name}: changed node ir_id"
                    );
                }
            }
        }
        let id = result
            .package
            .package_id()
            .expect("the package is identified");
        if name == "v2_nominal" {
            assert_eq!(id.digest.as_ref(), package_id, "{name}: package_id");
        }
        let bytes = result.package.canonical_bytes().expect("package bytes");
        if name == "v2_nominal" {
            assert_eq!(bytes.len(), length, "{name}: canonical byte length");
        }
        // The id is the SHA-256 of exactly those bytes, with no domain label.
        assert_eq!(sha256_hex(bytes), id.digest.as_ref(), "{name}");
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

/// Tracing: TC-048
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

/// Tracing: TC-048
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
    // `serde_json::to_value` is left in exactly the places that do not reach a
    // digest, a key, a preimage or a term order: the lossless-decode
    // comparison of the decoded wire (`v2/intake.rs`: the wire without its long
    // arrays, and each element of them), the first-difference pointer of the
    // reader (`v2/mod.rs`: the lock and preimage mirrors, the projection
    // comparison) and the literal-`type` comparison of an enum member body
    // (`v2/identity.rs`). Owner equality uses the typed values directly.
    // Any other call, or another in those files, fails.
    let expected: [(&str, usize); 3] =
        [("v2/mod.rs", 4), ("v2/intake.rs", 2), ("v2/identity.rs", 1)];
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

/// IR-274 part C: the output-mapping identity steps encode through
/// `quire-canonical` under the request byte limit. The production source of
/// `output_mapping.rs` holds no encoder of its own, no `serde_json` value or
/// call (so no `to_vec` or `to_value` reaches identity material), no
/// `u64::MAX` ceiling, and its three identity materials derive `FixedShape`.
///
/// Tracing: TC-048
/// ACs: FR-038-AC-91
#[trace("TC-048", "FR-038-AC-91")]
#[test]
fn tc_048_the_output_mapping_source_holds_no_encoder_and_no_unmetered_ceiling() {
    let text = fs::read_to_string(repository_path(
        "crates/quire-contract-model/src/output_mapping.rs",
    ))
    .expect("source reads");
    let production = production_source(&text);
    assert!(
        production.contains("fn identity_bytes<T>"),
        "the scan reads the production source"
    );
    for symbol in [
        "CanonicalWriter",
        "canonical_envelope_bytes",
        "digest_json",
        "serde_json_canonicalizer",
        "serde_json",
        "json!",
        "u64::MAX",
    ] {
        let hits = production.matches(symbol).count();
        assert_eq!(hits, 0, "output_mapping.rs: `{symbol}` x{hits}");
    }
    // The one ceiling chosen for identity material is the request byte limit;
    // any other spelling of a ceiling (`!0`, `MAX`, a literal) adds a second
    // `Limits::new` or changes this one. The unit test that spies on the
    // encoder's ceiling covers a value passed some other way.
    assert_eq!(
        production.matches("Limits::new(").count(),
        1,
        "output_mapping.rs builds exactly one encoder limit"
    );
    assert!(
        production.contains("Limits::new(limits.maximum_request_bytes)"),
        "the encoder limit is the request byte limit"
    );
    // Each step is called exactly once, with the caller's own limits (the
    // admission parameter `limits`, or the admitted request's `limits()`) and
    // `quire_canonical::to_vec` itself. A call that builds other limits or
    // wraps the encoder changes this text. Whitespace is collapsed so
    // rustfmt's line breaks do not matter.
    let flat = production.split_whitespace().collect::<Vec<_>>().join(" ");
    for call in [
        "request_identity_bytes(&request_material, &limits, quire_canonical::to_vec)",
        "record_identity_bytes(&material, request.limits(), quire_canonical::to_vec)",
        "package_identity_bytes(&material, request.limits(), quire_canonical::to_vec)",
    ] {
        assert_eq!(flat.matches(call).count(), 1, "call site `{call}`");
        let step = call.split('(').next().expect("step name");
        assert_eq!(
            flat.matches(&format!("{step}(")).count(),
            1,
            "`{step}` has no other call site"
        );
    }
    for material in [
        "RequestIdentityMaterial<'a>",
        "GeneratedOutputPackageIdentityMaterial<'a>",
        "MappingRecordIdentityMaterial<'a>",
    ] {
        let derive = format!("#[derive(Serialize, FixedShape)]\nstruct {material}");
        assert!(
            production.contains(&derive),
            "{material} derives FixedShape"
        );
    }
}

/// IR-274 part B: the v1 canonical objects and the bound identity envelope take
/// `quire-canonical` too. The production source of `canonical.rs` and
/// `binding.rs` holds no encoder of its own and no `serde_json` call that
/// builds bytes or a value, the one encoder call of each is
/// `quire_canonical::to_vec` or `quire_canonical::sha256` under its own
/// ceiling, `canonical_envelope_bytes` exists nowhere in the crate, and the
/// bound identity envelope derives `FixedShape` and holds no `Value`.
///
/// Tracing: TC-048
/// ACs: FR-038-AC-91, FR-038-AC-92
#[trace("TC-048", "FR-038-AC-91", "FR-038-AC-92")]
#[test]
fn tc_048_the_v1_canonical_and_binding_source_hold_no_encoder_of_their_own() {
    let source = |name: &str| {
        let text = fs::read_to_string(repository_path(&format!(
            "crates/quire-contract-model/src/{name}"
        )))
        .expect("source reads");
        production_source(&text)
    };
    let canonical = source("canonical.rs");
    let binding = source("binding.rs");
    assert!(
        canonical.contains("fn canonicalize<"),
        "the scan reads canonical.rs"
    );
    assert!(
        binding.contains("struct BoundIdentityEnvelope"),
        "the scan reads binding.rs"
    );
    for (name, text) in [("canonical.rs", &canonical), ("binding.rs", &binding)] {
        for symbol in [
            "CanonicalWriter",
            "canonical_envelope_bytes",
            "digest_json",
            "serde_json_canonicalizer",
            "value_to_vec",
            "serde_json::to_vec",
            "serde_json::to_value",
            "to_vec_pretty",
            "to_writer",
            "sha256_with_domain",
        ] {
            assert_eq!(text.matches(symbol).count(), 0, "{name}: `{symbol}`");
        }
        assert!(!text.contains("Encode for serde_json::Value"), "{name}");
        assert!(!text.contains("Encode for Value {"), "{name}");
        assert!(!text.contains("impl FixedShape for"), "{name}");
    }
    // `canonical.rs`: one encoder call, under the ceiling the caller passed,
    // and a digest over an explicit domain prefix.
    assert_eq!(canonical.matches("quire_canonical::to_vec(").count(), 1);
    assert_eq!(canonical.matches("Limits::new(").count(), 1);
    assert!(canonical.contains("Limits::new(maximum_bytes)"));
    assert!(canonical.contains("const DIGEST_DOMAIN: &[u8] = b\"quire-contract-ir\";"));
    // `binding.rs`: one hash of the envelope under the file-size ceiling.
    assert_eq!(binding.matches("quire_canonical::sha256(").count(), 1);
    assert_eq!(binding.matches("Limits::new(").count(), 1);
    assert!(binding.contains("Limits::new(MAX_CONFORMANCE_FILE_BYTES)"));
    // The bound identity envelope and its binding rows derive `FixedShape`, by
    // the derive and not by hand, and hold no `Value`.
    for item in ["BoundIdentityEnvelope<'a>", "BoundBindingIdentity<'a>"] {
        let derive = format!("#[derive(Serialize, FixedShape)]\nstruct {item}");
        assert!(binding.contains(&derive), "{item} derives FixedShape");
        let body = binding
            .split(&format!("struct {item} {{"))
            .nth(1)
            .and_then(|rest| rest.split("\n}").next())
            .expect("struct body");
        assert!(
            !body
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|token| token == "Value"),
            "{item} holds no Value"
        );
    }
    // `canonical_envelope_bytes` exists nowhere in the model crate's source.
    let mut files = Vec::new();
    rust_files_under(
        &repository_path("crates/quire-contract-model/src"),
        &mut files,
    );
    for path in files {
        let text = fs::read_to_string(&path).expect("source reads");
        for symbol in ["canonical_envelope_bytes", "CanonicalWriter", "digest_json"] {
            assert_eq!(
                text.matches(symbol).count(),
                0,
                "{}: `{symbol}`",
                path.display()
            );
        }
    }
}

/// Tracing: TC-048
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

/// Tracing: TC-048
/// ACs: FR-038-AC-95
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn tc_048_the_lowering_makes_no_expect_or_unwrap_on_an_encode() {
    let path = repository_path("crates/quire-contract-model/src/checked_package/v2/lower.rs");
    let text = fs::read_to_string(path).expect("source reads");
    let production = production_source(&text);
    assert!(production.contains("encode_package"));
    for call in [".expect(", ".unwrap(", ".unwrap_or_default(", "panic!("] {
        assert_eq!(production.matches(call).count(), 0, "`{call}` in lower.rs");
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-92
#[trace("TC-048", "FR-038-AC-92")]
#[test]
fn tc_048_preimages_take_the_path_their_depth_allows() {
    let sources = checked_package_sources();
    let all: String = sources.iter().map(|(_, text)| text.as_str()).collect();
    for encoded in [
        "impl Encode for ApplicationNodePreimage",
        "impl Encode for LoweredNodePreimage",
        "impl Encode for ContractPackagePreimage",
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
    for fixed in ["NominalIdentityPreimage", "StructuralPreimage"] {
        assert!(derives_fixed_shape(fixed), "{fixed} derives FixedShape");
    }
    for encoded in [
        "ApplicationNodePreimage",
        "LoweredNodePreimage",
        "ContractPackagePreimage",
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
