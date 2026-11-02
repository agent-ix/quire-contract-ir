// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-533: the V2 wire types encode to RFC 8785 canonical bytes through
//! `quire-canonical`, and the reader's canonical-bytes check is its bytes
//! (FR-038 "Canonical encoding of the wire types" and "Canonical bytes are
//! quire-canonical's bytes", FR-038-AC-74 through FR-038-AC-80).

use crate::support::checked_package::{
    canonical, evidence_for, family_key, index_fixture_node_ids, node_id,
    positive_operation_identities, refusal, refusal_bytes, settle, v2_all_families, v2_nominal,
    ExpectedRefusal,
};
use ix_trace_rs::trace;
use quire_canonical::{Encode, Error, FixedShape, LimitKind, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedCapabilityDisposition,
    CheckedDependencySelection, CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedNodeId,
    CheckedNodePathSegment, CheckedPackageLockV2, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedSelectionRole,
    CheckedSemanticGraphV2, CheckedSemanticNodeV2, CheckedSourceMapEntry, CheckedTerm,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The byte ceiling of every encode over an in-repo fixture.
const FIXTURE_CEILING: u64 = 1 << 24;

/// The in-repo positive package fixtures.
fn fixtures() -> Vec<(&'static str, Value)> {
    let mut fixtures = vec![
        ("v2_all_families", v2_all_families()),
        ("v2_nominal", v2_nominal()),
        (
            "positive_operation_identities",
            positive_operation_identities(),
        ),
    ];
    for (_, package) in &mut fixtures {
        index_fixture_node_ids(package);
    }
    fixtures
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
                "stage": "package_read", "code": "wrong_snapshot",
                "cause_tag": "revision-mismatch", "details": [], "loci": [],
            },
        ],
    }))
}

fn graph_of(package: &Value) -> CheckedSemanticGraphV2 {
    match CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    ) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted.graph().clone(),
        other => panic!("fixture must admit before its typed graph can be encoded: {other:?}"),
    }
}

fn raw_graph_of(package: &Value) -> CheckedSemanticGraphV2<Value> {
    typed(&package["semantic_graph"])
}

/// The envelope writes graph-local index operands, and refuses a structural
/// operand even though generic node-ID encoding retains that preimage form.
#[trace("TC-048", "FR-038-AC-377")]
#[test]
fn tc_048_envelope_writes_index_and_refuses_inline_node_path() {
    let package = v2_all_families();
    let lock: CheckedPackageLockV2 = typed(&package["lock"]);
    let mut graph = graph_of(&package);
    let source_map: Vec<CheckedSourceMapEntry> = typed_each(&package["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed_each(&package["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&package["diagnostics"]);
    let encode = |graph: &CheckedSemanticGraphV2| {
        encode_checked_package_v2(
            package["name"].as_str().expect("name"),
            &lock,
            graph,
            quire_contract_model::CheckedPackageSourceCorrespondenceRef {
                source_map: &source_map,
                refinement_step_rows: None,
            },
            quire_contract_model::CheckedPackageSidecarsRef {
                checked_term_data: None,
                temporal_bindings: None,
                schedulability_claims: None,
            },
            &capabilities,
            &diagnostics,
            Limits::new(FIXTURE_CEILING),
        )
    };

    graph.nodes[0]
        .node_id
        .path
        .push(CheckedNodePathSegment::Index(0));
    let encoded = encode(&graph).expect("index operand encodes");
    assert!(encoded
        .windows(b"{\"node\":0}".len())
        .any(|part| part == b"{\"node\":0}"));

    let structural = Arc::new(graph.nodes[1].node_id.clone());
    graph.nodes[0].node_id.path.pop();
    graph.nodes[0]
        .node_id
        .path
        .push(CheckedNodePathSegment::Node(structural));
    assert!(matches!(encode(&graph), Err(Error::Internal { .. })));
}

/// A dependency-reference node is declared, so even a syntactically valid
/// graph-local operand index cannot be published in that field.
#[trace("TC-048", "FR-038-AC-377")]
#[test]
fn tc_048_envelope_refuses_indexed_dependency_reference_node() {
    let package = v2_all_families();
    let lock: CheckedPackageLockV2 = typed(&package["lock"]);
    let mut graph = raw_graph_of(&package);
    let source_map: Vec<CheckedSourceMapEntry> = typed_each(&package["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed_each(&package["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&package["diagnostics"]);
    graph.nodes[0].body = json!({
        "term": "dependency_reference",
        "package": "acme/library",
        "node": {"package": "acme/library", "path": ["owned", {"node": 0}], "ordinal": 0}
    });
    let result = encode_checked_package_v2(
        package["name"].as_str().expect("name"),
        &lock,
        &graph,
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: None,
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            checked_term_data: None,
            temporal_bindings: None,
            schedulability_claims: None,
        },
        &capabilities,
        &diagnostics,
        Limits::new(FIXTURE_CEILING),
    );
    assert!(matches!(result, Err(Error::Internal { .. })));
}

// ---------------------------------------------------------------------------
// FR-038-AC-74: canonical closed envelope and forbidden old identity fields.
// ---------------------------------------------------------------------------

#[trace("TC-048", "FR-038-AC-74")]
#[test]
fn tc_048_closed_envelope_bytes_equal_independent_canonical_json() {
    for (name, package) in fixtures() {
        let lock: CheckedPackageLockV2 = typed(&package["lock"]);
        let graph = graph_of(&package);
        let source_map: Vec<CheckedSourceMapEntry> = typed_each(&package["source_map"]);
        let capabilities: Vec<CheckedCapability> = typed_each(&package["capability_report"]);
        let diagnostics: CheckedDiagnosticsV2 = typed(&package["diagnostics"]);
        let temporal_bindings: Option<Vec<quire_contract_model::TemporalBinding>> =
            package.get("temporal_bindings").map(typed_each);
        let encoded = encode_checked_package_v2(
            package["name"].as_str().expect("structural name"),
            &lock,
            &graph,
            quire_contract_model::CheckedPackageSourceCorrespondenceRef {
                source_map: &source_map,
                refinement_step_rows: None,
            },
            quire_contract_model::CheckedPackageSidecarsRef {
                checked_term_data: None,
                temporal_bindings: temporal_bindings.as_deref(),
            },
            &capabilities,
            &diagnostics,
            Limits::new(FIXTURE_CEILING),
        )
        .expect("closed envelope encodes");
        assert_eq!(encoded, canonical(&package), "{name}");
        for retired in ["identity_preimage", "identity_projection"] {
            let mut adverse = package.clone();
            adverse[retired] = json!({});
            let result = CheckedPackageV2::read(
                &canonical(&adverse),
                CheckedPackageReadLimits::bounded(),
                &evidence_for(&adverse),
            );
            assert_eq!(
                result,
                refusal(
                    CheckedPackageRefusalCode::UnknownMember,
                    &format!("/{retired}"),
                ),
                "{name}: retired field {retired}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// FR-038-AC-75: the graph.
// ---------------------------------------------------------------------------

/// A graph holding each nominal family, a `declaration`, a
/// `recursion_group` and bodies of every JSON kind, with the strings and
/// integers of FR-038-AC-75.
fn crafted_graph() -> CheckedSemanticGraphV2<Value> {
    let package = v2_nominal();
    let mut nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    let versions: BTreeSet<&str> = nodes
        .iter()
        .filter_map(|node| node["semantic_form"].as_str())
        .collect();
    assert_eq!(
        versions,
        BTreeSet::from(["dimension", "enum", "enum_value", "unit",]),
        "the nominal fixture carries each published family"
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
}

// ---------------------------------------------------------------------------
// FR-038-AC-76: the lock, source-map entry, capability, structural id and
// diagnostics.
// ---------------------------------------------------------------------------

/// A lock holding two dependency selections, with non-ASCII and astral
/// identities.
fn crafted_lock(package: &Value) -> CheckedPackageLockV2 {
    let mut lock = package["lock"].clone();
    lock["dependency_selections"] = json!([
        {"identity": "a/\u{e9}"},
        {"identity": "a/\u{1F600}"},
    ]);
    typed(&lock)
}

/// ACs: FR-038-AC-76
#[trace("TC-048", "FR-038-AC-76")]
#[test]
fn tc_048_lock_source_map_capability_node_id_and_diagnostics_bytes_equal_serde_json() {
    for (name, package) in fixtures() {
        assert_same_bytes(name, &typed::<CheckedPackageLockV2>(&package["lock"]));
        for entry in typed_each::<CheckedSourceMapEntry>(&package["source_map"]) {
            assert_same_bytes(name, &entry);
        }
        for capability in typed_each::<CheckedCapability>(&package["capability_report"]) {
            assert_same_bytes(name, &capability);
        }
        for node in package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
        {
            assert_same_bytes(name, &typed::<CheckedNodeId>(&node["node_id"]));
        }
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
    // Every capability disposition and a structural id with nested path.
    for disposition in CheckedCapabilityDisposition::ALL {
        assert_same_bytes(
            "capability",
            &typed::<CheckedCapability>(&json!({
                "feature": "f\u{e9}ature", "disposition": disposition.as_wire()
            })),
        );
    }
    assert_same_bytes(
        "structural id",
        &typed::<CheckedNodeId>(&json!({
            "package": "p/\u{e9}", "path": ["type", {"node": 0}], "ordinal": 1
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

impl Carrier for CheckedSemanticGraphV2<Value> {
    fn shallow() -> Self {
        let mut graph = raw_graph_of(&v2_all_families());
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

impl Carrier for CheckedDiagnosticsV2 {
    fn shallow() -> Self {
        let package = v2_all_families();
        typed(&json!({
            "catalog": package["diagnostics"]["catalog"],
            "entries": [{
                "stage": "package_read", "code": "wrong_snapshot",
                "cause_tag": "revision-mismatch", "details": [null],
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

/// ACs: FR-038-AC-77, FR-038-AC-78
#[trace("TC-048", "FR-038-AC-77", "FR-038-AC-78")]
#[test]
fn tc_048_a_graph_body_nested_100000_deep_encodes_on_a_256_kib_stack() {
    deep_encodes_and_stays_under_a_byte_ceiling::<CheckedSemanticGraphV2<Value>>();
}

/// ACs: FR-038-AC-77, FR-038-AC-78
#[trace("TC-048", "FR-038-AC-77", "FR-038-AC-78")]
#[test]
fn tc_048_diagnostic_details_nested_100000_deep_encode_on_a_256_kib_stack() {
    deep_encodes_and_stays_under_a_byte_ceiling::<CheckedDiagnosticsV2>();
}

#[trace("TC-451", "FR-051-AC-1", "FR-038-AC-77")]
#[test]
fn tc_451_typed_envelope_encodes_deep_details_on_a_small_stack() {
    let value = v2_all_families();
    let name = value["name"].as_str().expect("name").to_owned();
    let lock: CheckedPackageLockV2 = typed(&value["lock"]);
    let graph = graph_of(&value);
    let source_map: Vec<CheckedSourceMapEntry> = typed_each(&value["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed_each(&value["capability_report"]);
    let mut diagnostics = CheckedDiagnosticsV2::shallow();
    diagnostics.place(nested(DEEP));
    let bytes = on_small_stack(move || {
        let result = encode_checked_package_v2(
            &name,
            &lock,
            &graph,
            quire_contract_model::CheckedPackageSourceCorrespondenceRef {
                source_map: &source_map,
                refinement_step_rows: None,
            },
            quire_contract_model::CheckedPackageSidecarsRef {
                checked_term_data: None,
                temporal_bindings: None,
                schedulability_claims: None,
            },
            &capabilities,
            &diagnostics,
            Limits::new(u64::MAX),
        );
        let mut body = diagnostics.take();
        dismantle(&mut body);
        result
    })
    .expect("deep envelope encodes");
    let text = String::from_utf8(bytes).expect("canonical UTF-8");
    assert!(text.contains(&format!("\"details\":[{}]", nested_text(DEEP))));
}

#[trace("TC-451", "FR-051-AC-4")]
#[test]
fn tc_451_deep_closed_graph_encodes_once_with_exact_byte_limits() {
    let value = v2_all_families();
    let name = value["name"].as_str().expect("name").to_owned();
    let lock: CheckedPackageLockV2 = typed(&value["lock"]);
    let mut graph = raw_graph_of(&value);
    graph.nodes[0].body = nested(DEEP);
    let source_map: Vec<CheckedSourceMapEntry> = typed_each(&value["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed_each(&value["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&value["diagnostics"]);
    let (encoded, short) = on_small_stack(move || {
        let encode = |limit| {
            encode_checked_package_v2(
                &name,
                &lock,
                &graph,
                quire_contract_model::CheckedPackageSourceCorrespondenceRef {
                    source_map: &source_map,
                    refinement_step_rows: None,
                },
                quire_contract_model::CheckedPackageSidecarsRef {
                    checked_term_data: None,
                    temporal_bindings: None,
                    schedulability_claims: None,
                },
                &capabilities,
                &diagnostics,
                Limits::new(limit),
            )
        };
        let full = encode(u64::MAX).expect("deep graph encodes");
        assert_eq!(
            encode(u64::try_from(full.len()).expect("length")).expect("exact byte ceiling"),
            full
        );
        let short = encode(u64::try_from(full.len()).expect("length") - 1);
        dismantle(&mut graph.nodes[0].body);
        (full, short)
    });
    let text = String::from_utf8(encoded).expect("canonical UTF-8");
    assert_eq!(
        text.matches(&format!("\"body\":{}", nested_text(DEEP)))
            .count(),
        1
    );
    assert!(matches!(short, Err(Error::Limit(limit)) if limit.kind == LimitKind::CanonicalBytes));
}

// ---------------------------------------------------------------------------
// FR-038-AC-79: the reader's canonical-bytes check.
// ---------------------------------------------------------------------------

/// A fixture package whose first node holds `body`. The chosen body has no
/// semantic term, so canonical byte admission precedes its grammar refusal.
fn package_with_body(body: Value) -> Value {
    let mut package = positive_operation_identities();
    package["semantic_graph"]["nodes"][0]["body"] = body;
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

fn noncanonical() -> ExpectedRefusal {
    refusal_bytes(CheckedPackageRefusalCode::NoncanonicalWire)
}

fn invalid_body() -> ExpectedRefusal {
    refusal(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        "/semantic_graph/nodes/0/body",
    )
}

/// ACs: FR-038-AC-79
#[trace("TC-048", "FR-038-AC-79")]
#[test]
fn tc_048_an_integer_past_2_pow_53_a_whole_float_and_a_utf8_ordered_name_are_noncanonical() {
    // Each document also carries a invalid body and a body that is no
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
        assert_eq!(text.matches("\"n\":7777777").count(), 1);
        let bytes = text.replace("\"n\":7777777", &format!("\"n\":{spelled}"));
        assert_eq!(
            read_bytes(bytes.as_bytes(), &package),
            noncanonical(),
            "{spelled}"
        );
    }
    // A float that is not whole is canonical bytes: the document reaches the
    // body validation it also fails.
    let package = package_with_body(json!({"term": "bogus", "n": 1.5}));
    assert_eq!(read_bytes(&canonical(&package), &package), invalid_body());
}

/// The bytes of a package whose first node body holds the number text
/// `spelled` at `n`, written exactly as given, and the package.
fn package_bytes_with_number(spelled: &str) -> (Vec<u8>, Value) {
    let package = package_with_body(json!({"term": "bogus", "n": 7_777_777}));
    let text = String::from_utf8(canonical(&package)).expect("UTF-8");
    assert_eq!(text.matches("\"n\":7777777").count(), 1);
    let bytes = text.replace("\"n\":7777777", &format!("\"n\":{spelled}"));
    (bytes.into_bytes(), package)
}

/// ACs: FR-038-AC-111
#[trace("TC-048", "FR-038-AC-111")]
#[test]
fn tc_048_a_package_document_number_is_read_exactly_or_refused_without_a_pointer() {
    // A number whose exact value the document's bytes lose is noncanonical
    // wire with no pointer and no cause, before the invalid body the
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
        assert_eq!(read_bytes(&bytes, &package), invalid_body(), "{spelled}");
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

/// ACs: FR-038-AC-79
#[trace("TC-048", "FR-038-AC-79")]
#[test]
fn tc_048_the_canonical_bytes_of_those_values_are_not_refused_as_noncanonical() {
    // 9007199254740992 and `2` are canonical, so the document reaches the
    // body validation it also fails.
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
            invalid_body(),
            "{what}"
        );
    }
    // And with the identity re-derived and a literal body, they admit.
    let base = v2_all_families();
    let integer_type = node_id(&family_key("a3a3"));
    for literal in [
        json!(9_007_199_254_740_992_u64),
        json!(-9_007_199_254_740_992_i64),
        json!(2),
    ] {
        let mut package = base.clone();
        // Node 3 is the `value`/`literal` node; its changed body needs a new
        // structural key before the canonical document is read.
        package["semantic_graph"]["nodes"][3]["semantic_type"] = integer_type.clone();
        package["semantic_graph"]["nodes"][3]["dependencies"] = json!([integer_type]);
        package["semantic_graph"]["nodes"][3]["body"] = json!({
            "term": "literal", "type": integer_type, "value_kind": "integer", "value": literal
        });
        settle(&mut package);
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

/// ACs: FR-038-AC-79
#[trace("TC-048", "FR-038-AC-79")]
#[test]
fn tc_048_members_in_utf16_code_unit_order_pass_the_canonical_bytes_check() {
    let low = '\u{e000}';
    let high = '\u{10000}';
    let body = json!({"term": "bogus", "\u{e000}": 0, "\u{10000}": 0});
    let package = package_with_body(body);
    // The closed structural name and node ids are retained; the body grammar
    // decides this canonical document.
    let text = String::from_utf8(canonical(&package)).expect("UTF-8");
    let utf8_order = format!("\"{low}\":0,\"{high}\":0");
    let utf16_order = format!("\"{high}\":0,\"{low}\":0");
    assert_eq!(text.matches(&utf8_order).count(), 1, "single graph body");
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

/// The direct typed reader compares scalar tokens with the same canonical
/// writer used for identity bytes, including escaping and numeric spelling.
#[trace("TC-048", "FR-038-AC-214")]
#[test]
fn tc_048_direct_reader_refuses_noncanonical_scalar_spellings_before_a_stale_id() {
    let package = package_with_body(json!({"term": "bogus", "n": 1000, "text": "é"}));
    let canonical = String::from_utf8(canonical(&package)).expect("UTF-8");
    for (from, to) in [
        ("\"n\":1000", "\"n\":1e3"),
        ("\"n\":1000", "\"n\":-0"),
        ("\"text\":\"é\"", "\"text\":\"\\u00e9\""),
    ] {
        assert_eq!(canonical.matches(from).count(), 1, "single graph body");
        let bytes = canonical.replace(from, to);
        assert_eq!(
            read_bytes(bytes.as_bytes(), &package),
            noncanonical(),
            "{to}"
        );
    }
}

// ---------------------------------------------------------------------------
// FR-038-AC-80: where each type sits, read from the manifests and the source.
// ---------------------------------------------------------------------------

fn repository_file(path: &str) -> String {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&full).unwrap_or_else(|error| panic!("{}: {error}", full.display()))
}

/// AC-367 public API absence slice. Production sources contain no retired
/// semantic-id, candidate-key, stale-key or digest-pair API.
#[trace("TC-048", "FR-038-AC-367")]
#[test]
fn structural_v2_public_surface_has_no_stale_identity_api() {
    for path in files_under(&["crates/quire-contract-model/src", "src"], "rs") {
        let source = fs::read_to_string(&path).expect("checked-package source");
        let retired = retired_identity_identifiers(&source);
        assert!(
            retired.is_empty(),
            "{} contains retired identity identifiers: {retired:?}",
            path.display()
        );
    }
}

fn retired_identity_identifiers(source: &str) -> Vec<String> {
    const RETIRED: [&str; 5] = [
        "CheckedSemanticId",
        "candidate_key",
        "StaleNodeKey",
        "expected_digest",
        "actual_digest",
    ];

    struct Retired(Vec<String>);

    impl<'ast> syn::visit::Visit<'ast> for Retired {
        fn visit_item(&mut self, item: &'ast syn::Item) {
            let attrs = match item {
                syn::Item::Const(item) => &item.attrs,
                syn::Item::Enum(item) => &item.attrs,
                syn::Item::Fn(item) => &item.attrs,
                syn::Item::Impl(item) => &item.attrs,
                syn::Item::Macro(item) => &item.attrs,
                syn::Item::Mod(item) => &item.attrs,
                syn::Item::Static(item) => &item.attrs,
                syn::Item::Struct(item) => &item.attrs,
                syn::Item::Trait(item) => &item.attrs,
                syn::Item::Type(item) => &item.attrs,
                syn::Item::Use(item) => &item.attrs,
                _ => return syn::visit::visit_item(self, item),
            };
            if attrs.iter().any(|attr| {
                attr.path().is_ident("cfg")
                    && attr
                        .parse_args::<syn::Ident>()
                        .is_ok_and(|ident| ident == "test")
            }) {
                return;
            }
            syn::visit::visit_item(self, item);
        }

        fn visit_ident(&mut self, ident: &'ast syn::Ident) {
            let name = ident.to_string();
            if RETIRED.contains(&name.as_str()) {
                self.0.push(name);
            }
        }

        fn visit_macro(&mut self, macro_item: &'ast syn::Macro) {
            for token in macro_item
                .tokens
                .to_string()
                .split(|character: char| !character.is_alphanumeric() && character != '_')
            {
                if RETIRED.contains(&token) {
                    self.0.push(token.to_owned());
                }
            }
        }
    }

    let syntax = syn::parse_file(source).expect("production Rust syntax");
    let mut retired = Retired(Vec::new());
    syn::visit::Visit::visit_file(&mut retired, &syntax);
    retired.0
}

#[test]
fn structural_v2_absence_scan_catches_multiline_reexports_and_digest_pair() {
    let mutant = "pub use identity::{\n CheckedSemanticId,\n StaleNodeKey,\n};\n\
        pub struct OldReceipt { pub expected_digest: String, pub actual_digest: String }\n\
        pub fn candidate_key() {}";
    assert_eq!(
        retired_identity_identifiers(mutant),
        [
            "CheckedSemanticId",
            "StaleNodeKey",
            "expected_digest",
            "actual_digest",
            "candidate_key"
        ]
    );
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
            if !trimmed.starts_with("impl") {
                return None;
            }
            let rest = trimmed
                .split_once(" Encode for ")
                .or_else(|| trimmed.split_once(" quire_canonical::Encode for "))?
                .1;
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
const FIXED_DEPTH: [&str; 13] = [
    "CheckedCapability",
    "CheckedPackageLockV2",
    "CheckedOccurrenceRole",
    "CheckedSourceRegion",
    "CheckedArtifactRef",
    "CheckedSourceRef",
    "CheckedSelection",
    "CheckedDependencySelection",
    "CheckedOccurrence",
    "CheckedDeclaration",
    "CheckedDiagnosticStage",
    "CheckedDiagnosticCode",
    "CheckedDiagnosticCause",
];

/// The types that hold a `Value`, directly or through another type.
const HOLD_A_VALUE: [&str; 5] = [
    "CheckedSemanticGraphV2",
    "CheckedDiagnosticsV2",
    "CheckedSemanticNodeV2",
    "CheckedDiagnosticV2",
    "CheckedTerm",
];

const ENCODED: [&str; 3] = [
    "CheckedSemanticGraphV2",
    "CheckedDiagnosticsV2",
    "CheckedTerm",
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
    for retired in [
        "CheckedSemanticId",
        "CheckedPackageIdentityPreimageV2",
        "CheckedPackageIdentityPreimageRefV2",
    ] {
        assert!(!derived.contains(retired), "{retired} is retired");
        assert!(!encoded.contains(retired), "{retired} is retired");
    }
    // The closed vocabularies derive in the one macro that declares them.
    let vocabulary =
        repository_file("crates/quire-contract-model/src/checked_package/v2/vocabulary.rs");
    assert!(vocabulary.contains("::quire_canonical::FixedShape"));
}

/// ACs: FR-038-AC-80
#[trace("TC-048", "FR-038-AC-80")]
#[test]
fn tc_048_the_wire_types_are_on_the_path_the_compiler_sees() {
    assert_encode::<CheckedSourceMapEntry>();
    assert_fixed_shape::<CheckedCapability>();
    assert_fixed_shape::<CheckedCapabilityDisposition>();
    assert_fixed_shape::<CheckedPackageLockV2>();
    assert_fixed_shape::<CheckedSelectionRole>();
    assert_fixed_shape::<CheckedDependencySelection>();
    assert_encode::<CheckedNodeId>();
    assert_encode::<CheckedSemanticGraphV2>();
    assert_encode::<CheckedTerm>();
    assert_encode::<CheckedDiagnosticsV2>();
    const {
        assert!(is_fixed_shape!(CheckedPackageLockV2));
        assert!(!is_fixed_shape!(CheckedSourceMapEntry));
        assert!(!is_fixed_shape!(CheckedNodeId));
        assert!(!is_fixed_shape!(CheckedSemanticGraphV2));
        assert!(!is_fixed_shape!(CheckedDiagnosticsV2));
        assert!(!is_fixed_shape!(CheckedSemanticNodeV2));
        assert!(!is_fixed_shape!(CheckedDiagnosticV2));
    }
}

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
