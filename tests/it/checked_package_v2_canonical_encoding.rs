// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-533: the V2 wire types encode to RFC 8785 canonical bytes through
//! `quire-canonical`, and the reader's canonical-bytes check is its bytes
//! (FR-038 "Canonical encoding of the wire types" and "Canonical bytes are
//! quire-canonical's bytes", FR-038-AC-74 through FR-038-AC-80).

use crate::support::checked_package::{
    canonical, evidence_for, mint_ungrouped_structural_keys, positive_operation_identities,
    refresh_identity, refusal, refusal_bytes, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_canonical::{Encode, Error, FixedShape, LimitKind, Limits};
use quire_contract_ir::{
    CheckedCapability, CheckedCapabilityDisposition, CheckedDependencySelection,
    CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedNodeId, CheckedNodeProjectionV2,
    CheckedPackageIdentityPreimageV2, CheckedPackageLockV2, CheckedPackageReadLimits,
    CheckedPackageRefusal, CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedSelectionRole, CheckedSemanticGraphV2, CheckedSemanticId, CheckedSemanticNodeV2,
    CheckedSourceMapEntry,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// The byte ceiling of every encode over an in-repo fixture.
const FIXTURE_CEILING: u64 = 1 << 24;

/// The in-repo positive package fixtures.
fn fixtures() -> Vec<(&'static str, Value)> {
    vec![
        ("v2_all_families", v2_all_families()),
        ("v2_nominal", v2_nominal()),
        (
            "positive_operation_identities",
            positive_operation_identities(),
        ),
    ]
}

/// `value` as `serde_json` canonical bytes: `to_value` then `to_vec`, members
/// in sorted order.
fn serde_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(&serde_json::to_value(value).expect("serializable")).expect("bytes")
}

/// `value` as `quire-canonical` bytes.
fn quire_bytes<T: Encode>(value: &T) -> Vec<u8> {
    quire_canonical::to_vec(value, Limits::new(FIXTURE_CEILING)).expect("encodes")
}

/// Asserts, without printing either side, that both encodings agree.
fn assert_same_bytes<T: Encode + Serialize>(what: &str, value: &T) {
    let quire = quire_bytes(value);
    let serde = serde_bytes(value);
    assert!(
        quire == serde,
        "{what}: quire-canonical wrote {} bytes and serde_json {}",
        quire.len(),
        serde.len()
    );
}

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).expect("typed wire value")
}

fn typed_each<T: DeserializeOwned>(values: &Value) -> Vec<T> {
    values
        .as_array()
        .expect("array")
        .iter()
        .map(typed)
        .collect()
}

/// A body holding every JSON kind, strings with non-ASCII and astral
/// characters (U+00E9, U+1F600) and escapes, member names ordered the same by
/// UTF-8 bytes and by UTF-16 code units, and the integers at the 2^53 edge.
fn crafted_body() -> Value {
    json!({
        "term": "aggregate",
        "ascii": "plain",
        "é": "é",
        "\u{1F600}": "\u{1F600} \"quoted\" back\\slash\nline \u{1} \u{8}\u{c}\u{1f} \u{7f} \u{2028} /",
        "ints": [
            0, -1, 1, 9_007_199_254_740_992_i64, -9_007_199_254_740_992_i64,
            9_007_199_254_740_992_u64
        ],
        "nested": {"b": [true, false, null, "é"], "a": {"z": [], "y": {}, "x": [[], [{}]]}},
        "float": 1.5,
        "empty": "",
    })
}

fn first_region(package: &Value) -> Value {
    package["source_map"][0]["regions"][0].clone()
}

/// A diagnostics value whose entries hold nested objects and arrays in
/// `details` and whose `loci` are non-empty, over the fixture's catalog.
fn crafted_diagnostics(package: &Value) -> CheckedDiagnosticsV2 {
    let target = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    typed(&json!({
        "catalog": package["diagnostics"]["catalog"],
        "entries": [
            {
                "stage": "type_checking", "code": "invalid_package", "cause_tag": "invalid-value",
                "details": [
                    {"term": "literal", "type": target, "value_kind": "integer", "value": -1,
                     "é\u{1F600}": [1, {"a": null, "b": [true, false, "é"]}, []]},
                    [1, [2, [3, {"k": [{}]}]]],
                    "\u{1F600}", 9_007_199_254_740_992_i64, -9_007_199_254_740_992_i64, 0, null,
                    true, 1.5
                ],
                "loci": [first_region(package), first_region(package)],
            },
            {
                "stage": "package_read", "code": "stale_dependency",
                "cause_tag": "byte-digest-mismatch", "details": [], "loci": [],
            },
        ],
    }))
}

fn preimage_of(package: &Value) -> CheckedPackageIdentityPreimageV2 {
    typed(&package["identity_preimage"])
}

fn graph_of(package: &Value) -> CheckedSemanticGraphV2 {
    typed(&package["semantic_graph"])
}

/// Replaces `body` with `replacement` in the first projection of `preimage`.
fn with_projection_body(
    mut preimage: CheckedPackageIdentityPreimageV2,
    replacement: Value,
) -> CheckedPackageIdentityPreimageV2 {
    preimage.identity_projection[0].body = replacement;
    preimage
}

// ---------------------------------------------------------------------------
// FR-038-AC-74: the identity preimage and the package id.
// ---------------------------------------------------------------------------

/// Tracing: TC-048
/// ACs: FR-038-AC-74
#[trace("TC-048", "FR-038-AC-74")]
#[test]
fn tc_048_preimage_bytes_equal_serde_json_and_hash_to_the_package_id() {
    for (name, package) in fixtures() {
        let preimage = preimage_of(&package);
        assert_same_bytes(name, &preimage);
        let digest = quire_canonical::sha256(&preimage, Limits::new(FIXTURE_CEILING))
            .expect("hashes")
            .to_string();
        assert_eq!(
            Some(digest.as_str()),
            package["package_id"]["digest"].as_str(),
            "{name}: the package id is the SHA-256 of the canonical bytes, no domain label"
        );
        // The same bytes by a different route: hashing the written bytes.
        assert_eq!(
            digest,
            crate::support::checked_package::sha256_hex(&quire_bytes(&preimage)),
            "{name}"
        );
        // A domain-labelled digest is another hash: `package_id` takes none.
        let labelled = quire_canonical::sha256_with_domain(
            b"quire.package.semantic/v2",
            &preimage,
            Limits::new(FIXTURE_CEILING),
        )
        .expect("hashes")
        .to_string();
        assert_ne!(digest, labelled, "{name}");
        // Crafted projection bodies.
        let crafted = with_projection_body(preimage, crafted_body());
        assert_same_bytes(name, &crafted);
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-74
#[trace("TC-048", "FR-038-AC-74")]
#[test]
fn tc_048_a_preimage_that_differs_from_the_one_the_package_id_covers_is_stale() {
    for (name, package) in fixtures() {
        let evidence = evidence_for(&package);
        // One member of the preimage differs from the one `package_id` was
        // computed over: its first projection's `dependencies` gain an entry.
        let mut stale = package.clone();
        let extra = stale["semantic_graph"]["nodes"][0]["node_id"].clone();
        stale["identity_preimage"]["identity_projection"][0]["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .push(extra);
        match CheckedPackageV2::read(
            &canonical(&stale),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        ) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                refusal(
                    CheckedPackageRefusalCode::StaleDependency,
                    "/package_id/digest"
                ),
                "{name}"
            ),
            other => panic!("{name}: expected a stale refusal, read {other:?}"),
        }
        // The unchanged fixture still recomputes its recorded id.
        assert!(matches!(
            CheckedPackageV2::read(
                &canonical(&package),
                CheckedPackageReadLimits::bounded(),
                &evidence
            ),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
    }
}

// ---------------------------------------------------------------------------
// FR-038-AC-75: the graph.
// ---------------------------------------------------------------------------

/// A graph holding a node of each nominal preimage version, a `declaration`, a
/// `recursion_group` and bodies of every JSON kind, with the strings and
/// integers of FR-038-AC-75.
fn crafted_graph() -> CheckedSemanticGraphV2 {
    let package = v2_nominal();
    let mut nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    let versions: BTreeSet<&str> = nodes
        .iter()
        .filter_map(|node| node["nominal_identity_preimage"]["version"].as_str())
        .collect();
    assert_eq!(
        versions,
        BTreeSet::from([
            "quire.dimension-node/v1",
            "quire.enum-declaration-node/v1",
            "quire.enum-member-node/v1",
            "quire.unit-node/v1",
        ]),
        "the nominal fixture carries a preimage of each version"
    );
    let mut template = nodes[0].clone();
    template["recursion_group"] = json!("group-é\u{1F600}");
    for (position, body) in [
        crafted_body(),
        json!(null),
        json!(true),
        json!(false),
        json!("é\u{1F600}"),
        json!(0),
        json!(-1),
        json!(9_007_199_254_740_992_i64),
        json!(-9_007_199_254_740_992_i64),
        json!([]),
        json!({}),
        json!([[[]], {"a": [{"b": {}}]}]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut node = template.clone();
        node["body"] = body;
        node["occurrences"] = json!([{"role": "expression", "ordinal": position}]);
        nodes.push(node);
    }
    let mut graph = package["semantic_graph"].clone();
    graph["nodes"] = Value::Array(nodes);
    typed(&graph)
}

/// Tracing: TC-048
/// ACs: FR-038-AC-75
#[trace("TC-048", "FR-038-AC-75")]
#[test]
fn tc_048_graph_bytes_equal_serde_json_for_every_fixture_and_crafted_graph() {
    for (name, package) in fixtures() {
        assert_same_bytes(name, &graph_of(&package));
    }
    let graph = crafted_graph();
    assert!(graph.nodes.iter().any(|node| node.declaration.is_some()));
    assert!(graph
        .nodes
        .iter()
        .any(|node| node.recursion_group.is_some()));
    assert_same_bytes("crafted graph", &graph);
    // An absent optional member is omitted, never written as `null`.
    let bytes = String::from_utf8(quire_bytes(&graph_of(&v2_all_families()))).expect("UTF-8");
    assert!(!bytes.contains("\"recursion_group\":null"));
    assert!(!bytes.contains("\"declaration\":null"));
    assert!(!bytes.contains("\"nominal_identity_preimage\":null"));
}

// ---------------------------------------------------------------------------
// FR-038-AC-76: the lock, source-map entry, capability, semantic id and
// diagnostics.
// ---------------------------------------------------------------------------

/// A lock holding a model selection and two dependency selections, with
/// non-ASCII and astral identities.
fn crafted_lock(package: &Value) -> CheckedPackageLockV2 {
    let mut lock = package["lock"].clone();
    lock["model_selections"] = json!([{
        "identity": "dom\u{e9}ine/\u{1F600}", "digest_domain": "sha256-jcs",
        "digest": "5".repeat(64),
    }]);
    lock["dependency_selections"] = json!([
        {"identity": "a/\u{e9}", "package_id": {
            "domain": "quire.package.semantic/v2", "algorithm": "sha256", "digest": "3".repeat(64)}},
        {"identity": "a/\u{1F600}", "package_id": {
            "domain": "quire.package.semantic/v2", "algorithm": "sha256", "digest": "4".repeat(64)}},
    ]);
    typed(&lock)
}

/// Tracing: TC-048
/// ACs: FR-038-AC-76
#[trace("TC-048", "FR-038-AC-76")]
#[test]
fn tc_048_lock_source_map_capability_semantic_id_and_diagnostics_bytes_equal_serde_json() {
    for (name, package) in fixtures() {
        assert_same_bytes(name, &typed::<CheckedPackageLockV2>(&package["lock"]));
        for entry in typed_each::<CheckedSourceMapEntry>(&package["source_map"]) {
            assert_same_bytes(name, &entry);
        }
        for capability in typed_each::<CheckedCapability>(&package["capability_report"]) {
            assert_same_bytes(name, &capability);
        }
        assert_same_bytes(name, &typed::<CheckedSemanticId>(&package["package_id"]));
        assert_same_bytes(
            name,
            &typed::<CheckedDiagnosticsV2>(&package["diagnostics"]),
        );
        assert_same_bytes(name, &crafted_lock(&package));
        assert_same_bytes(name, &crafted_diagnostics(&package));
    }
    let package = v2_all_families();
    // The crafted diagnostics carry what AC-76 names.
    let diagnostics = crafted_diagnostics(&package);
    assert!(diagnostics
        .entries
        .iter()
        .any(|entry| !entry.loci.is_empty() && entry.details.iter().any(Value::is_object)));
    assert!(diagnostics
        .entries
        .iter()
        .any(|entry| entry.details.iter().any(Value::is_array)));
    // Every capability disposition and a semantic id of any digest.
    for disposition in CheckedCapabilityDisposition::ALL {
        assert_same_bytes(
            "capability",
            &typed::<CheckedCapability>(&json!({
                "feature": "f\u{e9}ature", "disposition": disposition.as_wire()
            })),
        );
    }
    assert_same_bytes(
        "semantic id",
        &typed::<CheckedSemanticId>(&json!({
            "domain": "d\u{e9}", "algorithm": "sha256", "digest": "0".repeat(64)
        })),
    );
}

// ---------------------------------------------------------------------------
// FR-038-AC-77 and FR-038-AC-78: a body nested as deep as its input.
// ---------------------------------------------------------------------------

/// A wire type that holds one `Value` the encode must write iteratively.
trait Carrier: Encode + Serialize + Clone + Send + 'static {
    /// The value with its one `Value` `null`.
    fn shallow() -> Self;
    /// Puts `body` where the `Value` is.
    fn place(&mut self, body: Value);
    /// Takes the `Value` back out, so it can be dropped without recursion.
    fn take(&mut self) -> Value;
    /// The text around the `Value` in the `serde_json` text of
    /// [`Carrier::shallow`].
    const AROUND: (&'static str, &'static str);
}

impl Carrier for CheckedSemanticGraphV2 {
    fn shallow() -> Self {
        let mut graph = graph_of(&v2_all_families());
        graph.nodes.truncate(1);
        graph.nodes[0].body = Value::Null;
        graph
    }
    fn place(&mut self, body: Value) {
        self.nodes[0].body = body;
    }
    fn take(&mut self) -> Value {
        std::mem::take(&mut self.nodes[0].body)
    }
    const AROUND: (&'static str, &'static str) = ("\"body\":", "");
}

impl Carrier for CheckedPackageIdentityPreimageV2 {
    fn shallow() -> Self {
        let mut preimage = preimage_of(&v2_all_families());
        preimage.identity_projection.truncate(1);
        preimage.identity_projection[0].body = Value::Null;
        preimage
    }
    fn place(&mut self, body: Value) {
        self.identity_projection[0].body = body;
    }
    fn take(&mut self) -> Value {
        std::mem::take(&mut self.identity_projection[0].body)
    }
    const AROUND: (&'static str, &'static str) = ("\"body\":", "");
}

impl Carrier for CheckedDiagnosticsV2 {
    fn shallow() -> Self {
        let package = v2_all_families();
        typed(&json!({
            "catalog": package["diagnostics"]["catalog"],
            "entries": [{
                "stage": "package_read", "code": "stale_dependency",
                "cause_tag": "byte-digest-mismatch", "details": [null],
                "loci": [first_region(&package)],
            }],
        }))
    }
    fn place(&mut self, body: Value) {
        self.entries[0].details[0] = body;
    }
    fn take(&mut self) -> Value {
        std::mem::take(&mut self.entries[0].details[0])
    }
    const AROUND: (&'static str, &'static str) = ("\"details\":[", "]");
}

/// `depth` arrays around one `null`, built without recursion.
fn nested(depth: usize) -> Value {
    let mut value = Value::Null;
    for _ in 0..depth {
        value = Value::Array(vec![value]);
    }
    value
}

/// The text of [`nested`], written out by repetition.
fn nested_text(depth: usize) -> String {
    format!("{}null{}", "[".repeat(depth), "]".repeat(depth))
}

/// Empties `value`, dropping its descendants iteratively: a deep value's own
/// drop recurses once per level.
fn dismantle(value: &mut Value) {
    let mut pending = vec![std::mem::take(value)];
    while let Some(mut next) = pending.pop() {
        match &mut next {
            Value::Array(elements) => pending.append(elements),
            Value::Object(members) => {
                pending.extend(
                    std::mem::take(members)
                        .into_iter()
                        .map(|(_, member)| member),
                );
            }
            _ => {}
        }
    }
}

/// The canonical text of `T` with `body_text` where its `Value` is, spliced
/// into the `serde_json` text of the shallow value rather than written by
/// `serde_json`.
fn expected_text<T: Carrier>(body_text: &str) -> String {
    let (before, after) = T::AROUND;
    let shallow = String::from_utf8(serde_bytes(&T::shallow())).expect("UTF-8 text");
    let slot = format!("{before}null{after}");
    assert_eq!(shallow.matches(&slot).count(), 1, "{shallow}");
    shallow.replace(&slot, &format!("{before}{body_text}{after}"))
}

/// Runs `work` on a thread whose stack is 256 KiB: a native recursion over a
/// value this deep overflows it, and the process aborts.
fn on_small_stack<R: Send + 'static>(work: impl FnOnce() -> R + Send + 'static) -> R {
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(work)
        .expect("thread spawns")
        .join()
        .expect("the thread completes")
}

/// Encodes `T` holding `body` under `ceiling` on a 256 KiB stack, then drops
/// `body` without recursion.
fn encode_on_small_stack<T: Carrier>(body: Value, ceiling: u64) -> Result<Vec<u8>, Error> {
    on_small_stack(move || {
        let mut carrier = T::shallow();
        carrier.place(body);
        let encoded = quire_canonical::to_vec(&carrier, Limits::new(ceiling));
        let mut body = carrier.take();
        dismantle(&mut body);
        encoded
    })
}

const DEEP: usize = 100_000;
const PAST_THE_READERS_CEILING: usize = 20_000;

fn deep_encodes_and_stays_under_a_byte_ceiling<T: Carrier>() {
    let expected = expected_text::<T>(&nested_text(DEEP)).into_bytes();
    // FR-038-AC-77: the nesting encodes, and the bytes are the expected text.
    let bytes = encode_on_small_stack::<T>(nested(DEEP), u64::MAX).expect("encodes");
    assert!(
        bytes == expected,
        "{} bytes against {}",
        bytes.len(),
        expected.len()
    );
    // FR-038-AC-78: a ceiling of the exact length returns the bytes.
    let length = u64::try_from(expected.len()).expect("length");
    let exact = encode_on_small_stack::<T>(nested(DEEP), length).expect("the exact ceiling");
    assert!(exact == expected);
    // One byte lower returns the byte-limit error and no bytes.
    match encode_on_small_stack::<T>(nested(DEEP), length - 1) {
        Err(Error::Limit(limit)) => {
            assert_eq!(limit.kind, LimitKind::CanonicalBytes);
            assert_eq!(limit.bound, length - 1);
        }
        Err(other) => panic!("expected the byte-limit error, got {other}"),
        Ok(bytes) => panic!("expected no bytes, got {}", bytes.len()),
    }
    // A body past both the reader's default depth and its ceiling encodes.
    let past = encode_on_small_stack::<T>(nested(PAST_THE_READERS_CEILING), u64::MAX)
        .expect("a 20000-level body is not refused for its depth");
    assert!(past == expected_text::<T>(&nested_text(PAST_THE_READERS_CEILING)).into_bytes());
    // An integer past 2^53 is the encoder's refusal, naming the value.
    for (value, named) in [
        (9_007_199_254_740_993_i128, json!(9_007_199_254_740_993_u64)),
        (
            -9_007_199_254_740_993_i128,
            json!(-9_007_199_254_740_993_i64),
        ),
    ] {
        match encode_on_small_stack::<T>(json!([[named]]), u64::MAX) {
            Err(Error::IntegerMagnitudeAboveMaximum(refused)) => assert_eq!(refused, value),
            Err(other) => panic!("expected the integer refusal, got {other}"),
            Ok(bytes) => panic!("expected no bytes, got {}", bytes.len()),
        }
    }
    // The integers at the edge are written.
    let edge = encode_on_small_stack::<T>(json!([9_007_199_254_740_992_i64]), u64::MAX)
        .expect("2^53 encodes");
    assert!(edge == expected_text::<T>("[9007199254740992]").into_bytes());
}

/// Tracing: TC-048
/// ACs: FR-038-AC-77, FR-038-AC-78
#[trace("TC-048", "FR-038-AC-77", "FR-038-AC-78")]
#[test]
fn tc_048_a_graph_body_nested_100000_deep_encodes_on_a_256_kib_stack() {
    deep_encodes_and_stays_under_a_byte_ceiling::<CheckedSemanticGraphV2>();
}

/// Tracing: TC-048
/// ACs: FR-038-AC-77, FR-038-AC-78
#[trace("TC-048", "FR-038-AC-77", "FR-038-AC-78")]
#[test]
fn tc_048_a_preimage_projection_nested_100000_deep_encodes_on_a_256_kib_stack() {
    deep_encodes_and_stays_under_a_byte_ceiling::<CheckedPackageIdentityPreimageV2>();
}

/// Tracing: TC-048
/// ACs: FR-038-AC-77, FR-038-AC-78
#[trace("TC-048", "FR-038-AC-77", "FR-038-AC-78")]
#[test]
fn tc_048_diagnostic_details_nested_100000_deep_encode_on_a_256_kib_stack() {
    deep_encodes_and_stays_under_a_byte_ceiling::<CheckedDiagnosticsV2>();
}

// ---------------------------------------------------------------------------
// FR-038-AC-79: the reader's canonical-bytes check.
// ---------------------------------------------------------------------------

/// A fixture package whose first node and first projection hold `body`. Its
/// `package_id` is left as recorded, so the document also earns the stale
/// refusal, and its body is no semantic term, so it also earns a grammar one.
fn package_with_body(body: Value) -> Value {
    let mut package = positive_operation_identities();
    package["semantic_graph"]["nodes"][0]["body"] = body.clone();
    package["identity_preimage"]["identity_projection"][0]["body"] = body;
    package
}

fn read_bytes(bytes: &[u8], package: &Value) -> CheckedPackageRefusal {
    match CheckedPackageV2::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn noncanonical() -> CheckedPackageRefusal {
    refusal_bytes(CheckedPackageRefusalCode::NoncanonicalWire)
}

fn stale() -> CheckedPackageRefusal {
    refusal(
        CheckedPackageRefusalCode::StaleDependency,
        "/package_id/digest",
    )
}

/// Tracing: TC-048
/// ACs: FR-038-AC-79
#[trace("TC-048", "FR-038-AC-79")]
#[test]
fn tc_048_an_integer_past_2_pow_53_a_whole_float_and_a_utf8_ordered_name_are_noncanonical() {
    // Each document also carries a stale package id and a body that is no
    // semantic term; the canonical-bytes refusal comes first, with no pointer.
    let past = json!({"term": "bogus", "n": 9_007_199_254_740_993_u64});
    let past_negative = json!({"term": "bogus", "n": -9_007_199_254_740_993_i64});
    let whole_float = json!({"term": "bogus", "n": 2.0});
    // serde_json writes the members of an object in UTF-8 byte order: U+E000
    // (EE 80 80) before U+10000 (F0 90 80 80).
    let utf8_order = json!({"term": "bogus", "\u{e000}": 0, "\u{10000}": 0});
    for (what, body) in [
        ("past 2^53", past),
        ("past -2^53", past_negative),
        ("a whole float", whole_float),
        ("UTF-8 order", utf8_order),
    ] {
        let package = package_with_body(body);
        let bytes = canonical(&package);
        assert_eq!(read_bytes(&bytes, &package), noncanonical(), "{what}");
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-79
#[trace("TC-048", "FR-038-AC-79")]
#[test]
fn tc_048_an_integer_spelled_past_the_64_bit_range_is_noncanonical() {
    // serde_json reads these as floats whose RFC 8785 text is the same
    // digits, so the byte comparison alone would admit them; they are still
    // integers past 2^53 (FR-038). Each document also earns a stale refusal.
    for spelled in [
        "100000000000000000000",
        "-100000000000000000000",
        "18446744073709552000",
        "-9223372036854777000",
    ] {
        let package = package_with_body(json!({"term": "bogus", "n": 7_777_777}));
        let text = String::from_utf8(canonical(&package)).expect("UTF-8");
        assert_eq!(text.matches("\"n\":7777777").count(), 2);
        let bytes = text.replace("\"n\":7777777", &format!("\"n\":{spelled}"));
        assert_eq!(
            read_bytes(bytes.as_bytes(), &package),
            noncanonical(),
            "{spelled}"
        );
    }
    // A float that is not whole is canonical bytes: the document reaches the
    // package id comparison it also fails.
    let package = package_with_body(json!({"term": "bogus", "n": 1.5}));
    assert_eq!(read_bytes(&canonical(&package), &package), stale());
}

/// The bytes of a package whose first node body holds the number text
/// `spelled` at `n`, written exactly as given, and the package.
fn package_bytes_with_number(spelled: &str) -> (Vec<u8>, Value) {
    let package = package_with_body(json!({"term": "bogus", "n": 7_777_777}));
    let text = String::from_utf8(canonical(&package)).expect("UTF-8");
    assert_eq!(text.matches("\"n\":7777777").count(), 2);
    let bytes = text.replace("\"n\":7777777", &format!("\"n\":{spelled}"));
    (bytes.into_bytes(), package)
}

/// Tracing: TC-048
/// ACs: FR-038-AC-111
#[trace("TC-048", "FR-038-AC-111")]
#[test]
fn tc_048_a_package_document_number_is_read_exactly_or_refused_without_a_pointer() {
    // A number whose exact value the document's bytes lose is noncanonical
    // wire with no pointer and no cause, before the stale package id the
    // document also earns.
    for spelled in ["0.1000000000000000000001", "9007199254740993.5"] {
        let (bytes, package) = package_bytes_with_number(spelled);
        assert_eq!(read_bytes(&bytes, &package), noncanonical(), "{spelled}");
    }
    // Each is its double's shortest round-trip text, which `serde_json` reads
    // as a neighbouring double without `float_roundtrip`. With the feature on
    // it is not noncanonical, and the document reaches the package id
    // comparison it also fails. This loop checks the build as a whole: other
    // crates (quire-canonical's `serde_json` feature) also turn the feature
    // on, so it does not fail if this crate's manifest drops it. The manifest
    // assertion below is the oracle for AC-111's declaration clause.
    for spelled in [
        "0.1",
        "1.2793061557049685",
        "1.2106592671318679",
        "1.3567384036451073",
    ] {
        let (bytes, package) = package_bytes_with_number(spelled);
        assert_eq!(read_bytes(&bytes, &package), stale(), "{spelled}");
    }
    // The crate that holds the reader declares the feature itself, so the
    // exact read does not depend on what other crates in the build turn on.
    let manifest = repository_file("crates/quire-contract-model/Cargo.toml");
    let line = manifest
        .lines()
        .find(|line| line.starts_with("serde_json"))
        .expect("the manifest declares serde_json");
    assert!(line.contains("\"float_roundtrip\""), "{line}");
}

/// Tracing: TC-048
/// ACs: FR-038-AC-79
#[trace("TC-048", "FR-038-AC-79")]
#[test]
fn tc_048_the_canonical_bytes_of_those_values_are_not_refused_as_noncanonical() {
    // 9007199254740992 and `2` are canonical, so the document reaches the
    // package id comparison it also fails.
    for (what, body) in [
        (
            "2^53",
            json!({"term": "bogus", "n": 9_007_199_254_740_992_u64}),
        ),
        (
            "-2^53",
            json!({"term": "bogus", "n": -9_007_199_254_740_992_i64}),
        ),
        ("2", json!({"term": "bogus", "n": 2})),
    ] {
        let package = package_with_body(body);
        assert_eq!(
            read_bytes(&canonical(&package), &package),
            stale(),
            "{what}"
        );
    }
    // And with the identity re-derived and a literal body, they admit.
    let base = positive_operation_identities();
    let self_type = base["semantic_graph"]["nodes"][0]["node_id"].clone();
    for literal in [
        json!(9_007_199_254_740_992_u64),
        json!(-9_007_199_254_740_992_i64),
        json!(2),
    ] {
        let mut package = base.clone();
        // Node 2 is the `value`/`literal` node; its changed body needs a new
        // structural key before the canonical document is read.
        package["semantic_graph"]["nodes"][2]["body"] = json!({
            "term": "literal", "type": self_type, "value_kind": "integer", "value": literal
        });
        mint_ungrouped_structural_keys(&mut package);
        refresh_identity(&mut package);
        assert!(
            matches!(
                CheckedPackageV2::read(
                    &canonical(&package),
                    CheckedPackageReadLimits::bounded(),
                    &evidence_for(&package),
                ),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{literal}"
        );
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-79
#[trace("TC-048", "FR-038-AC-79")]
#[test]
fn tc_048_members_in_utf16_code_unit_order_pass_the_canonical_bytes_check() {
    let low = '\u{e000}';
    let high = '\u{10000}';
    let body = json!({"term": "bogus", "\u{e000}": 0, "\u{10000}": 0});
    let mut package = package_with_body(body);
    // The package id the reader recomputes: quire-canonical's digest of the
    // preimage, so the document gets past the identity comparison.
    package["package_id"]["digest"] = json!(quire_canonical::sha256(
        &preimage_of(&package),
        Limits::new(FIXTURE_CEILING)
    )
    .expect("hashes")
    .to_string());
    let text = String::from_utf8(canonical(&package)).expect("UTF-8");
    let utf8_order = format!("\"{low}\":0,\"{high}\":0");
    let utf16_order = format!("\"{high}\":0,\"{low}\":0");
    assert_eq!(text.matches(&utf8_order).count(), 2, "graph and projection");
    let bytes = text.replace(&utf8_order, &utf16_order).into_bytes();
    // Not canonical-bytes-refused; the closed body grammar decides the
    // document.
    assert_eq!(
        read_bytes(&bytes, &package),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "/semantic_graph/nodes/0/body"
        )
    );
    // The same document in UTF-8 order is the one refused.
    assert_eq!(read_bytes(text.as_bytes(), &package), noncanonical());
}

// ---------------------------------------------------------------------------
// FR-038-AC-80: where each type sits, read from the manifests and the source.
// ---------------------------------------------------------------------------

fn repository_file(path: &str) -> String {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&full).unwrap_or_else(|error| panic!("{}: {error}", full.display()))
}

/// Every file under `directories` (relative to the repository root) with the
/// extension `extension`, found with an explicit stack.
fn files_under(directories: &[&str], extension: &str) -> Vec<PathBuf> {
    let mut pending: Vec<PathBuf> = directories
        .iter()
        .map(|directory| Path::new(env!("CARGO_MANIFEST_DIR")).join(directory))
        .collect();
    let mut found = Vec::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(
                fs::read_dir(&path)
                    .expect("directory reads")
                    .map(|entry| entry.expect("directory entry").path()),
            );
        } else if path.extension().is_some_and(|found| found == extension) {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// The crate sources: the model crate and the root bridge.
fn crate_sources() -> Vec<(PathBuf, String)> {
    files_under(&["crates/quire-contract-model/src", "src"], "rs")
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).expect("source reads");
            (path, text)
        })
        .collect()
}

/// The identifier after `keyword` on `line`, if the line declares one.
fn declared_name<'l>(line: &'l str, keywords: &[&str]) -> Option<&'l str> {
    keywords.iter().find_map(|keyword| {
        let rest = line
            .trim_start()
            .strip_prefix("pub ")
            .unwrap_or(line.trim_start());
        let rest = rest.strip_prefix(keyword)?;
        let end = rest
            .find(|character: char| !(character.is_alphanumeric() || character == '_'))
            .unwrap_or(rest.len());
        Some(&rest[..end])
    })
}

/// The types with `#[derive(.., FixedShape)]` in `source`.
fn derived_fixed_shape(source: &str) -> BTreeSet<String> {
    let mut derived = BTreeSet::new();
    let mut pending = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[derive(") && trimmed.contains("FixedShape") {
            pending = true;
        } else if pending {
            if let Some(name) = declared_name(line, &["struct ", "enum "]) {
                derived.insert(name.to_owned());
                pending = false;
            }
        }
    }
    derived
}

/// The types with an `impl Encode for` in `source`.
fn implemented_encode(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed
                .strip_prefix("impl Encode for ")
                .or_else(|| trimmed.strip_prefix("impl quire_canonical::Encode for "))?;
            let end = rest
                .find(|character: char| !(character.is_alphanumeric() || character == '_'))
                .unwrap_or(rest.len());
            Some(rest[..end].to_owned())
        })
        .collect()
}

/// The fixed-depth types FR-038 names, by role: the artifact-reference types
/// are `CheckedArtifactRef` and `CheckedSourceRef` (FR-038-AC-46 through
/// FR-038-AC-61).
const FIXED_DEPTH: [&str; 25] = [
    "CheckedSemanticId",
    "CheckedSourceMapEntry",
    "CheckedCapability",
    "CheckedPackageLockV2",
    "CheckedNodeId",
    "CheckedOccurrenceRole",
    "CheckedSourceRegion",
    "CheckedArtifactRef",
    "CheckedSourceRef",
    "CheckedSelection",
    "CheckedDomainPackageRef",
    "CheckedDependencySelection",
    "CheckedOccurrence",
    "CheckedDeclaration",
    "NominalIdentityPreimage",
    "EnumDeclarationPreimage",
    "EnumMemberPreimage",
    "DimensionPreimage",
    "UnitPreimage",
    "NominalOwner",
    "DimensionTerm",
    "CheckedRational",
    "CheckedDiagnosticStage",
    "CheckedDiagnosticCode",
    "CheckedDiagnosticCause",
];

/// The types that hold a `Value`, directly or through another type.
const HOLD_A_VALUE: [&str; 6] = [
    "CheckedPackageIdentityPreimageV2",
    "CheckedSemanticGraphV2",
    "CheckedDiagnosticsV2",
    "CheckedNodeProjectionV2",
    "CheckedSemanticNodeV2",
    "CheckedDiagnosticV2",
];

const ENCODED: [&str; 3] = [
    "CheckedPackageIdentityPreimageV2",
    "CheckedSemanticGraphV2",
    "CheckedDiagnosticsV2",
];

fn assert_fixed_shape<T: FixedShape>() {}

fn assert_encode<T: Encode>() {}

/// Whether `T` implements `FixedShape`, decided at compile time for a concrete
/// `T`: the inherent constant exists only where the bound holds and wins over
/// the trait's default where it does.
struct Probe<T>(PhantomData<T>);

trait NotFixedShape {
    const IS_FIXED_SHAPE: bool = false;
}

impl<T> NotFixedShape for Probe<T> {}

impl<T: FixedShape> Probe<T> {
    const IS_FIXED_SHAPE: bool = true;
}

macro_rules! is_fixed_shape {
    ($type:ty) => {
        <Probe<$type>>::IS_FIXED_SHAPE
    };
}

/// Tracing: TC-048
/// ACs: FR-038-AC-80
#[trace("TC-048", "FR-038-AC-80")]
#[test]
fn tc_048_quire_canonical_is_a_branch_main_git_dependency_with_its_source_allowed() {
    for manifest in ["Cargo.toml", "crates/quire-contract-model/Cargo.toml"] {
        let text = repository_file(manifest);
        let line = text
            .lines()
            .find(|line| line.starts_with("quire-canonical"))
            .unwrap_or_else(|| panic!("{manifest} declares quire-canonical"));
        assert!(
            line.contains("git = \"https://github.com/agent-ix/quire-canonical\""),
            "{manifest}: {line}"
        );
        assert!(line.contains("branch = \"main\""), "{manifest}: {line}");
        for other in ["rev =", "tag =", "path ="] {
            assert!(!line.contains(other), "{manifest}: {line}");
        }
    }
    let deny = repository_file("deny.toml");
    let allowed = deny
        .split("allow-git")
        .nth(1)
        .expect("deny.toml has an allow-git list");
    assert!(
        allowed.contains("\"https://github.com/agent-ix/quire-canonical\""),
        "{allowed}"
    );
    let lock = repository_file("Cargo.lock");
    assert_eq!(
        lock.matches("name = \"quire-canonical\"").count(),
        1,
        "one lock entry"
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-80
#[trace("TC-048", "FR-038-AC-80")]
#[test]
fn tc_048_fixed_depth_types_derive_and_value_holding_types_implement_encode() {
    let sources = crate_sources();
    let mut derived = BTreeSet::new();
    let mut encoded = BTreeSet::new();
    for (path, text) in &sources {
        derived.extend(derived_fixed_shape(text));
        encoded.extend(implemented_encode(text));
        for line in text.lines() {
            let trimmed = line.trim_start();
            // No hand-written `FixedShape`, no `DEPTH` literal.
            assert!(
                !(trimmed.starts_with("impl") && trimmed.contains("FixedShape for")),
                "{}: {line}",
                path.display()
            );
            assert!(!line.contains("const DEPTH"), "{}: {line}", path.display());
            // No wrapper or alias around a `quire-canonical` type.
            let declares = declared_name(line, &["struct ", "enum ", "type "]).is_some();
            assert!(
                !(declares && line.contains("quire_canonical")),
                "{}: {line}",
                path.display()
            );
            if declares && trimmed.contains('(') {
                for wrapped in ["Writer", "Limits", "Sha256Digest", "Document", "Sink"] {
                    assert!(!trimmed.contains(wrapped), "{}: {line}", path.display());
                }
            }
        }
    }
    for name in FIXED_DEPTH {
        assert!(derived.contains(name), "{name} derives FixedShape");
        assert!(!encoded.contains(name), "{name} has no hand-written Encode");
    }
    for name in HOLD_A_VALUE {
        assert!(!derived.contains(name), "{name} holds a Value");
    }
    for name in ENCODED {
        assert!(encoded.contains(name), "{name} implements Encode");
    }
    // The closed vocabularies derive in the one macro that declares them.
    let vocabulary =
        repository_file("crates/quire-contract-model/src/checked_package/v2/vocabulary.rs");
    assert!(vocabulary.contains("::quire_canonical::FixedShape"));
}

/// Tracing: TC-048
/// ACs: FR-038-AC-80
#[trace("TC-048", "FR-038-AC-80")]
#[test]
fn tc_048_the_wire_types_are_on_the_path_the_compiler_sees() {
    assert_fixed_shape::<CheckedSemanticId>();
    assert_fixed_shape::<CheckedSourceMapEntry>();
    assert_fixed_shape::<CheckedCapability>();
    assert_fixed_shape::<CheckedCapabilityDisposition>();
    assert_fixed_shape::<CheckedPackageLockV2>();
    assert_fixed_shape::<CheckedSelectionRole>();
    assert_fixed_shape::<CheckedDependencySelection>();
    assert_fixed_shape::<CheckedNodeId>();
    assert_encode::<CheckedPackageIdentityPreimageV2>();
    assert_encode::<CheckedSemanticGraphV2>();
    assert_encode::<CheckedDiagnosticsV2>();
    const {
        assert!(is_fixed_shape!(CheckedPackageLockV2));
        assert!(is_fixed_shape!(CheckedSourceMapEntry));
        assert!(!is_fixed_shape!(CheckedPackageIdentityPreimageV2));
        assert!(!is_fixed_shape!(CheckedSemanticGraphV2));
        assert!(!is_fixed_shape!(CheckedDiagnosticsV2));
        assert!(!is_fixed_shape!(CheckedNodeProjectionV2));
        assert!(!is_fixed_shape!(CheckedSemanticNodeV2));
        assert!(!is_fixed_shape!(CheckedDiagnosticV2));
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-80
#[trace("TC-048", "FR-038-AC-80")]
#[test]
fn tc_048_no_copy_of_quire_canonical_is_in_the_repository() {
    // No copy of its published vectors, wherever they are.
    let mut pending = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR"))];
    while let Some(path) = pending.pop() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if matches!(
                name,
                "target" | ".git" | "worktrees" | ".worktrees" | "node_modules"
            ) {
                continue;
            }
            pending.extend(
                fs::read_dir(&path)
                    .expect("directory reads")
                    .map(|entry| entry.expect("directory entry").path()),
            );
        } else {
            assert_ne!(name, "jcs-vectors.json", "{}", path.display());
        }
    }
    // No declaration of its encoder, writer or shape trait in the crate source.
    for (path, text) in crate_sources() {
        for declaration in [
            "pub struct Writer",
            "pub trait FixedShape",
            "pub trait Encode",
        ] {
            assert!(
                !text.contains(declaration),
                "{}: {declaration}",
                path.display()
            );
        }
    }
}
