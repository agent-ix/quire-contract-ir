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
    CompleteLoweringProfileV2, CompleteLoweringRecordV2, NominalIdentityPreimage,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// `ir_id` of every lowered node, then the lowered package's `package_id` and
/// canonical byte length, recorded from the lowering before the move to
/// `quire-canonical`. One `node ir_id` pair per line, in graph order.
const RECORDED_ALL_FAMILIES: &str = "\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 39fcdcad8558e6266381e0473c730c66965087f81bcb7adc5f99aa0155514af9
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb d66cb5b15dff6ae1cec8f875535ee8fffa8935fef9d2a0d118c5ca6172f6bbd7
cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc 69b26b6562dccae6f97679d338438e70d77445fb83527b30b49d7c6601c15111
dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd 103e55988e513f1c600574dfdc83f88e73266935600961f50611ad07408570aa
eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee 62330a59c40bffdba5f0d89a83b837271563c223f6b68101976c463508c9c48d
ace54dacfc17d1530436ee8074e3b14ed40e087033115595bb8d1d392b12b36e 0ea17a9e943e5e051d3b4f033777f46f6848d8026da4aaaa0ab3c8b4e1d5eeaa
1010101010101010101010101010101010101010101010101010101010101010 616ba6525b290c84f1c407dc708b910a35ec307cc9615532a55389c28c8116c6
2020202020202020202020202020202020202020202020202020202020202020 17f858cfdadae63b3919bdbfd51be1f3931a8b26c7214b99347be08db8fdb07b
3030303030303030303030303030303030303030303030303030303030303030 8a140681fe45569f9a58af68dcced609b321023d909dcf436f017bf64c6eafa8
5f934c5208321e5e08835d4506c1d7ad736a25c3ab6ee8415daa842c0ff587ba d990ad3092d1d88fd811752064b70d02232b9ea656273e104f8e3e4fe9a1f395
ab48836daadf2308e60c9ace4f273245980b53ede1e65416cc00270ccae71e79 93fd0a1b39ae451798c67eecd3ed9e1de35504702fecf2118fbaf1f796bb17a3
773d33e42cbc9e5f3735330b41fd06b6c58609eb2369a408b7801b87104a035d 95ba07af33eae01930fcaee8b03bfd70722e78c870dbf7d061fc4604936e1721
7070707070707070707070707070707070707070707070707070707070707070 55e635a70d36a40c3cdb033996ffd501e720c12b0adee9db2828fe86fd228e3c
8080808080808080808080808080808080808080808080808080808080808080 914140e10534e3f22803ecbb40b89c215e43f419211abb23fcbe36cd205762d9
1515151515151515151515151515151515151515151515151515151515151515 8771428969ce08b4d8373f7cb077a691ddf041b89ce54e4bb18cb9e6030628ec
1616161616161616161616161616161616161616161616161616161616161616 69d1fac2393973f8af0eaade013e84292a178c7647e473ab39a1ab782114e1e7
f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0f4a0 bd74d12307f1dee12e77dec4a898135a73d625b8c38daad0ad8cd8e01fda9914
a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1 7bc463ef2cd246231a48e0f812baed64d2f61d8e724ca466bc0b53b0e24f91b2
9e1bda145090a3ee6217e5549dbfa47351221847ae2382bbf07ea6a5bdb5556e fe3eac164599910357b08ce94b5d0b04ad079949c11f51ffde5d2da7090ae4ae
ac694939ba336aab1fe50613ed2326dcf27c1cd69f477474a82331e3b089ef7b 7525871ceff187be476bea1fd57486c634f88221902ea57e58dbb53d6381050d
a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6 8d0ddd66b582ce9c631a2dc91485ad771512cd9af4592018328a3bd5cc51695c
a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3 1765f6c8ec87d21d7709370998acac0eed95a5280682091e7f25a53e88342709
a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4a4 25284da55bcd9dddc4785e94661131b679f57a103ead665f0a2fa0a560ae2dc8
";
const ALL_FAMILIES_PACKAGE: (&str, usize) = (
    "d688cce71d20310c7c018499420aedfe4247808e2eb10d689c4226cc47453aba",
    39522,
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
beefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef 86b877fd505f970ee50918aeed93cb1496ebfda80fb1a655244249cb8312454b
cafecafecafecafecafecafecafecafecafecafecafecafecafecafecafecafe d884a10a9189123760eb4e54d447d585b1bdc6021c045b7f2dcf26ee0a7c0025
d00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00dd00d 689ce9939e0be390e0ef90f045992aa51fd7ea385e7e026a7d7663bc556025ff
f00df00df00df00df00df00df00df00df00df00df00df00df00df00df00df00d 32084934e99105f13caf8377bcff6fcf2445aa6147e5fd6532f44fbcd35bfbea
91d352432696da426e7324426094c63a0bcfa0f74bc22880132a004bbae62081 9547532a7a7b33376907f037d05e66bad03fe14d5c85a667ace93e249f1d0981
";
const OPERATIONS_PACKAGE: (&str, usize) = (
    "c716dbac37d2c2560fb67137a0628677bc44156bfe59e9c5e9c2179df1c2576a",
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
        assert_eq!(lowered, expected, "{name}: `ir_id` values");
        let id = result
            .package
            .package_id()
            .expect("the package is identified");
        assert_eq!(id.digest.as_ref(), package_id, "{name}: package_id");
        let bytes = result.package.canonical_bytes().expect("package bytes");
        assert_eq!(bytes.len(), length, "{name}: canonical byte length");
        // The id is the SHA-256 of exactly those bytes, with no domain label.
        assert_eq!(sha256_hex(bytes), package_id, "{name}");
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

fn repository_path(relative: &str) -> PathBuf {
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
fn production_source(text: &str) -> String {
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

/// Every source file under `checked_package/`, without its `#[cfg(test)]`
/// items and without the out-of-line test files.
fn checked_package_sources() -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    rust_files_under(
        &repository_path("crates/quire-contract-model/src/checked_package"),
        &mut files,
    );
    files
        .into_iter()
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name != "tests.rs" && name != "ceiling_tests.rs")
        })
        .map(|path| {
            let text = fs::read_to_string(&path).expect("source reads");
            let production = production_source(&text);
            (path, production)
        })
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
    // comparison and the first-difference pointer of the reader (`v2/mod.rs`:
    // the decoded wire, the lock and preimage mirrors, the projection
    // comparison) and the literal-`type` comparison of an enum member body
    // (`v2/identity.rs`). Any other call, or another in those files, fails.
    let expected: [(&str, usize); 2] = [("v2/mod.rs", 5), ("v2/identity.rs", 1)];
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
