//! The closed-schema decode of the package document, without native recursion
//! following the nesting of a term.
//!
//! A node `body` and a diagnostic's `details`
//! entry may nest as deep as the strict parser's limit of 128. The outer typed
//! decode borrows each such term as a raw source slice, then decodes it into a
//! `Value` only after that outer frame has returned. A `NodeId.path` contains
//! fixed-depth string or graph-index segments, so a derived chain does not
//! increase JSON nesting depth. Deep terms stay within the reader stack budget.
//!
//! The lossless-decode check (no member defaulted, nulled or dropped) compares
//! borrowed source array items against their typed serialization one at a
//! time, except structural node identities, which compare from raw slices.

use super::{
    CheckedDeclaration, CheckedDiagnosticCause, CheckedDiagnosticCode, CheckedDiagnosticStage,
    CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedNodeOwner, CheckedPackageEnvelope,
    CheckedPackageLockV2, RawCheckedPackageWireV2, RawSemanticGraphV2, RawSemanticNodeV2,
    ValidationFailure,
};
use crate::checked_package::common::{decode_closed_bytes, node_pointer, strict_parse};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedCapability, CheckedNodeId, CheckedOccurrence, CheckedSourceMapEntry,
    CheckedSourceRegion,
};
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use serde_json::Value;
use std::collections::BTreeMap;

/// QSpec's KeyOrdinal admits a JSON integer through 2^53-1, or a canonical
/// decimal string strictly above that bound.
#[ix_trace_rs::string_edge]
fn key_ordinal(value: &Value) -> bool {
    const MAX_SAFE: &str = "9007199254740991";
    if let Some(number) = value.as_u64() {
        return number <= 9_007_199_254_740_991;
    }
    let Some(text) = value.as_str() else {
        return false;
    };
    !text.is_empty()
        && !text.starts_with('0')
        && text.bytes().all(|byte| byte.is_ascii_digit())
        && (text.len() > MAX_SAFE.len() || (text.len() == MAX_SAFE.len() && text > MAX_SAFE))
}

/// Check a present position ordinal before semantic graph validation.
fn position_member_shape(body: &Value) -> bool {
    let mut pending = vec![body];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(items) => pending.extend(items),
            Value::Object(map) => {
                if map.get("term").and_then(Value::as_str) == Some("application") {
                    if let Some(member) = map.get("operation").and_then(|op| op.get("member")) {
                        if member.get("kind").and_then(Value::as_str) == Some("position") {
                            let Some(fields) = member.as_object() else {
                                return false;
                            };
                            if fields.contains_key("position")
                                && !fields.get("position").is_some_and(key_ordinal)
                            {
                                return false;
                            }
                        }
                    }
                }
                pending.extend(map.values());
            }
            _ => {}
        }
    }
    true
}

// The borrowed intake mirrors only the closed containers that contain an
// arbitrary JSON term. Those terms are decoded after the outer typed wire
// frame has returned, keeping their recursion off its call stack. Every other
// member is decoded by the same public type as the admitted wire.
type BorrowedWire<'a> = CheckedPackageEnvelope<
    Box<str>,
    Box<str>,
    CheckedPackageLockV2,
    BorrowedGraph<'a>,
    Vec<CheckedSourceMapEntry>,
    Vec<super::refinement::CheckedRefinementStepRowRegion>,
    super::checked_term_data::CheckedTermDataV2,
    Vec<super::temporal_binding::TemporalBinding>,
    Vec<super::CheckedSchedulabilityClaimRow>,
    Vec<CheckedCapability>,
    BorrowedDiagnostics<'a>,
>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedGraph<'a> {
    graph_version: Box<str>,
    #[serde(borrow)]
    nodes: Vec<BorrowedNode<'a>>,
}

// Keep native wire errors at the logical ID boundary. The refinement below
// then locates an invalid ordinal or path segment without exposing an
// implementation-dependent Serde member path. The public ID deserializer
// remains the one owner of the flat ID grammar.
struct WireNodeId(CheckedNodeId);

impl<'de> Deserialize<'de> for WireNodeId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = <&RawValue>::deserialize(deserializer)?;
        serde_json::from_str(raw.get())
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedNode<'a> {
    node_id: WireNodeId,
    schema_version: Box<str>,
    node_tag: Box<str>,
    semantic_form: Box<str>,
    semantic_type: WireNodeId,
    dependencies: Vec<WireNodeId>,
    occurrences: Vec<CheckedOccurrence>,
    #[serde(default)]
    recursion_group: Option<Box<str>>,
    #[serde(default, deserialize_with = "present_nullable")]
    declaration: Option<Option<CheckedDeclaration>>,
    #[serde(default)]
    owner: Option<CheckedNodeOwner>,
    #[serde(default)]
    owner_model: Option<WireNodeId>,
    #[serde(borrow)]
    body: &'a RawValue,
}

// The outer option records member presence; the inner option records JSON null.
fn present_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedDiagnostics<'a> {
    catalog: CheckedArtifactRef,
    #[serde(borrow)]
    entries: Vec<BorrowedDiagnostic<'a>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedDiagnostic<'a> {
    stage: CheckedDiagnosticStage,
    code: CheckedDiagnosticCode,
    cause_tag: CheckedDiagnosticCause,
    #[serde(borrow)]
    details: Vec<&'a RawValue>,
    loci: Vec<CheckedSourceRegion>,
}

#[ix_trace_rs::string_edge]
pub(super) fn decode_borrowed_wire(
    bytes: &[u8],
) -> Result<RawCheckedPackageWireV2, ValidationFailure> {
    let wire: BorrowedWire<'_> = decode_closed_bytes(bytes).map_err(|mut failure| {
        if let ValidationFailure::Refused(ref mut refusal) = failure {
            if refusal.code == CheckedPackageRefusalCode::UnknownMember {
                if let Some(index) = refusal
                    .primary_path()
                    .and_then(|at| at.as_str().strip_prefix("/schedulability_claims/"))
                    .and_then(|suffix| suffix.split('/').next())
                    .and_then(|index| index.parse::<usize>().ok())
                {
                    refusal.code = CheckedPackageRefusalCode::MalformedWire;
                    refusal.set_primary_path(Some(
                        JsonPointer::root()
                            .key("schedulability_claims")
                            .index(index),
                    ));
                }
            }
            if refusal.code == CheckedPackageRefusalCode::MalformedWire {
                if let Some(at) = refusal.primary_path().cloned() {
                    if let Some(member) = invalid_node_id_field(bytes, &at) {
                        refusal.set_primary_path(Some(member));
                    } else if let Some(member) = invalid_clock_field(bytes, &at) {
                        refusal.set_primary_path(Some(member));
                    }
                }
            }
        }
        failure
    })?;
    if wire
        .schedulability_claims
        .as_ref()
        .is_some_and(Vec::is_empty)
    {
        return Err(malformed(JsonPointer::root().key("schedulability_claims")));
    }
    let graph = wire.semantic_graph;
    let diagnostics = wire.diagnostics;
    let mut nodes = Vec::with_capacity(graph.nodes.len());
    for (position, node) in graph.nodes.into_iter().enumerate() {
        if node.node_tag.as_ref() == "scalar_type"
            && node.semantic_form.as_ref() == "dimension_equivalence"
            && node.declaration.is_some()
        {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                node_pointer(position),
            ));
        }
        let body = strict_parse(node.body.get().as_bytes())?;
        if !position_member_shape(&body) {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                node_pointer(position),
            ));
        }
        nodes.push(RawSemanticNodeV2 {
            node_id: node.node_id.0,
            schema_version: node.schema_version,
            node_tag: node.node_tag,
            semantic_form: node.semantic_form,
            semantic_type: node.semantic_type.0,
            dependencies: node.dependencies.into_iter().map(|id| id.0).collect(),
            occurrences: node.occurrences,
            recursion_group: node.recursion_group,
            declaration: node.declaration.flatten(),
            owner: node.owner,
            owner_model: node.owner_model.map(|id| id.0),
            body,
        });
    }
    Ok(CheckedPackageEnvelope {
        contract_version: wire.contract_version,
        name: wire.name,
        lock: wire.lock,
        semantic_graph: RawSemanticGraphV2 {
            graph_version: graph.graph_version,
            nodes,
        },
        source_map: wire.source_map,
        refinement_step_rows: wire.refinement_step_rows,
        checked_term_data: wire.checked_term_data,
        temporal_bindings: wire.temporal_bindings,
        schedulability_claims: wire.schedulability_claims,
        capability_report: wire.capability_report,
        diagnostics: CheckedDiagnosticsV2 {
            catalog: diagnostics.catalog,
            entries: diagnostics
                .entries
                .into_iter()
                .map(|entry| {
                    Ok(CheckedDiagnosticV2 {
                        stage: entry.stage,
                        code: entry.code,
                        cause_tag: entry.cause_tag,
                        details: entry
                            .details
                            .into_iter()
                            .map(|term| strict_parse(term.get().as_bytes()))
                            .collect::<Result<Vec<_>, ValidationFailure>>()?,
                        loci: entry.loci,
                    })
                })
                .collect::<Result<Vec<_>, ValidationFailure>>()?,
        },
    })
}

#[ix_trace_rs::string_edge]
fn invalid_clock_field(bytes: &[u8], at: &JsonPointer) -> Option<JsonPointer> {
    if !at.as_str().starts_with("/temporal_bindings/") || !at.as_str().ends_with("/clock") {
        return None;
    }
    let clock: Value = serde_json::from_str(source_raw(bytes, at)?).ok()?;
    let field = match clock.get("kind")?.as_str()? {
        "event_position" => "sequence_authority",
        "timestamped_event" => "timestamp_unit",
        "fixed_sample" => "unit",
        _ => return None,
    };
    clock
        .get(field)
        .filter(|value| !value.is_string())
        .map(|_| at.clone().key(field))
}

/// Refine a buffered node-ID decode error to an ordinal or path segment.
/// V2 path objects are closed graph-local indices; nested IDs are refused at
/// the segment itself, before semantic-graph admission.
#[ix_trace_rs::string_edge]
fn invalid_node_id_field(bytes: &[u8], at: &JsonPointer) -> Option<JsonPointer> {
    const MAX_SAFE: u64 = 9_007_199_254_740_991;
    let members: BTreeMap<&str, &RawValue> = serde_json::from_str(source_raw(bytes, at)?).ok()?;
    if !members.contains_key("package") || !members.contains_key("path") {
        return None;
    }
    if let Some(ordinal) = members.get("ordinal") {
        if serde_json::from_str::<u64>(ordinal.get())
            .ok()
            .is_none_or(|value| value > MAX_SAFE)
        {
            return Some(at.clone().key("ordinal"));
        }
    }
    let segments: Vec<&RawValue> = serde_json::from_str(members.get("path")?.get()).ok()?;
    for (index, segment) in segments.into_iter().enumerate() {
        let raw = segment.get();
        if raw.starts_with('"') {
            continue;
        }
        if raw.starts_with('{') {
            let object: BTreeMap<&str, &RawValue> = serde_json::from_str(raw).ok()?;
            if object.len() == 1
                && object.get("node").is_some_and(|value| {
                    serde_json::from_str::<u64>(value.get())
                        .ok()
                        .is_some_and(|value| value <= MAX_SAFE)
                })
            {
                continue;
            }
        }
        return Some(at.clone().key("path").index(index));
    }
    None
}

/// Read only the JSON value named by `pointer`. Borrowed raw subtrees retain
/// slices of the caller's bytes; no ancestor is decoded into a `Value`.
pub(super) fn source_value(bytes: &[u8], pointer: &JsonPointer) -> Option<Value> {
    serde_json::from_str(source_raw(bytes, pointer)?).ok()
}

/// Locate a source value without deserializing its ancestors. A package can
/// contain a valid term near serde_json's depth limit even when the requested
/// member is shallow; deserializing the root as RawValue would reject it.
fn source_raw<'a>(bytes: &'a [u8], pointer: &JsonPointer) -> Option<&'a str> {
    let mut current = bytes;
    let path = pointer.as_str();
    if !path.is_empty() {
        for token in path.strip_prefix('/')?.split('/') {
            let key = token.replace("~1", "/").replace("~0", "~");
            current = match current.first()? {
                b'{' => object_member(current, &key)?,
                b'[' => array_item(current, key.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
    }
    std::str::from_utf8(current).ok()
}

/// The strict and canonical scans precede every source lookup. These local
/// scans only delimit a child span and do not materialize skipped subtrees.
fn object_member<'a>(object: &'a [u8], key: &str) -> Option<&'a [u8]> {
    let mut at = 1;
    while object.get(at) != Some(&b'}') {
        let end = value_end(object, at)?;
        let member: &str = serde_json::from_slice(object.get(at..end)?).ok()?;
        at = end;
        if object.get(at) != Some(&b':') {
            return None;
        }
        at += 1;
        let end = value_end(object, at)?;
        if member == key {
            return object.get(at..end);
        }
        at = end;
        match object.get(at)? {
            b',' => at += 1,
            b'}' => return None,
            _ => return None,
        }
    }
    None
}

fn array_item(array: &[u8], target: usize) -> Option<&[u8]> {
    let mut at = 1;
    let mut index = 0;
    while array.get(at) != Some(&b']') {
        let end = value_end(array, at)?;
        if index == target {
            return array.get(at..end);
        }
        at = end;
        if array.get(at) != Some(&b',') {
            return None;
        }
        at += 1;
        index += 1;
    }
    None
}

/// End of one already validated JSON value, found with a heap stack.
pub(super) fn value_end(bytes: &[u8], start: usize) -> Option<usize> {
    let first = *bytes.get(start)?;
    if !matches!(first, b'{' | b'[' | b'"') {
        let mut at = start;
        while !matches!(bytes.get(at), None | Some(b',' | b'}' | b']')) {
            at += 1;
        }
        return Some(at);
    }
    let mut at = start;
    let mut closers = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    loop {
        let byte = *bytes.get(at)?;
        at += 1;
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
                if closers.is_empty() {
                    return Some(at);
                }
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => closers.push(b'}'),
            b'[' => closers.push(b']'),
            b'}' | b']' => {
                if closers.pop() != Some(byte) {
                    return None;
                }
                if closers.is_empty() {
                    return Some(at);
                }
            }
            _ => {}
        }
    }
}

/// `malformed_wire` at `at`.
fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

/// Check the lossless closed decode against borrowed slices of the source.
/// The comparator walks borrowed object members and array items as the typed
/// serializer visits them. Neither a root field nor one large array element
/// becomes a second `Value` tree. Arbitrary terms already came directly from
/// their raw source slices and are skipped by this closed-member check.
pub(super) fn check_lossless_source(
    bytes: &[u8],
    wire: &RawCheckedPackageWireV2,
) -> Result<(), ValidationFailure> {
    let root: BTreeMap<String, &RawValue> =
        serde_json::from_slice(bytes).map_err(|_| malformed(JsonPointer::root()))?;
    let nodes = raw_array(
        raw_member(root.get("semantic_graph").copied(), "nodes"),
        &["semantic_graph", "nodes"],
    )?;
    let source_map = raw_array(root.get("source_map").copied(), &["source_map"])?;
    let diagnostic_entries = raw_array(
        raw_member(root.get("diagnostics").copied(), "entries"),
        &["diagnostics", "entries"],
    )?;

    // Root member order matches `serde_json::Value`'s BTreeMap order used by
    // the prior first-difference check. Within each object, the comparator
    // likewise chooses the first differing source member rather than the
    // serializer's declaration order.
    compare_root(&root, "capability_report", &wire.capability_report, &[])?;
    compare_root(&root, "contract_version", &wire.contract_version, &[])?;
    compare_root(&root, "diagnostics", &wire.diagnostics, &["entries"])?;
    compare_root(&root, "lock", &wire.lock, &[])?;
    compare_root(&root, "name", &wire.name, &[])?;
    compare_root(&root, "semantic_graph", &wire.semantic_graph, &["nodes"])?;
    if nodes.len() != wire.semantic_graph.nodes.len() {
        return Err(malformed(
            JsonPointer::root().key("semantic_graph").key("nodes"),
        ));
    }
    for (index, (original, node)) in nodes.iter().zip(&wire.semantic_graph.nodes).enumerate() {
        let at = JsonPointer::root()
            .key("semantic_graph")
            .key("nodes")
            .index(index);
        super::lossless::compare(
            original,
            node,
            at.clone(),
            &[
                "body",
                "node_id",
                "semantic_type",
                "dependencies",
                "owner_model",
            ],
        )?;
        compare_id_member(original, "node_id", &node.node_id, at.clone())?;
        compare_id_member(original, "semantic_type", &node.semantic_type, at.clone())?;
        let dependencies: Vec<&RawValue> = serde_json::from_str(
            raw_member(Some(original), "dependencies")
                .ok_or_else(|| malformed(at.clone().key("dependencies")))?
                .get(),
        )
        .map_err(|_| malformed(at.clone().key("dependencies")))?;
        if dependencies.len() != node.dependencies.len() {
            return Err(malformed(at.clone().key("dependencies")));
        }
        for (item, (raw, id)) in dependencies.iter().zip(&node.dependencies).enumerate() {
            compare_id(raw, id, at.clone().key("dependencies").index(item))?;
        }
        match (&node.owner_model, raw_member(Some(original), "owner_model")) {
            (Some(id), Some(raw)) => compare_id(raw, id, at.key("owner_model"))?,
            (None, None) => {}
            _ => return Err(malformed(at.key("owner_model"))),
        }
    }
    if source_map.len() != wire.source_map.len() {
        return Err(malformed(JsonPointer::root().key("source_map")));
    }
    for (index, (original, entry)) in source_map.iter().zip(&wire.source_map).enumerate() {
        let at = JsonPointer::root().key("source_map").index(index);
        super::lossless::compare(original, entry, at.clone(), &["node_id"])?;
        compare_id_member(original, "node_id", &entry.node_id, at)?;
    }
    if let Some(rows) = &wire.refinement_step_rows {
        let originals = raw_array(
            root.get("refinement_step_rows").copied(),
            &["refinement_step_rows"],
        )?;
        if originals.len() != rows.len() {
            return Err(malformed(JsonPointer::root().key("refinement_step_rows")));
        }
        for (index, (original, row)) in originals.iter().zip(rows).enumerate() {
            let at = JsonPointer::root().key("refinement_step_rows").index(index);
            super::lossless::compare(original, row, at.clone(), &["node_id"])?;
            compare_id_member(original, "node_id", &row.node_id, at)?;
        }
    } else if root.contains_key("refinement_step_rows") {
        return Err(malformed(JsonPointer::root().key("refinement_step_rows")));
    }
    if let Some(data) = &wire.checked_term_data {
        compare_root(&root, "checked_term_data", data, &[])?;
    } else if root.contains_key("checked_term_data") {
        return Err(malformed(JsonPointer::root().key("checked_term_data")));
    }
    if let Some(rows) = &wire.temporal_bindings {
        let originals = raw_array(
            root.get("temporal_bindings").copied(),
            &["temporal_bindings"],
        )?;
        if rows.is_empty() || originals.len() != rows.len() {
            return Err(malformed(JsonPointer::root().key("temporal_bindings")));
        }
        for (index, (original, row)) in originals.iter().zip(rows).enumerate() {
            let at = JsonPointer::root().key("temporal_bindings").index(index);
            super::lossless::compare(original, row, at.clone(), &["declaration"])?;
            compare_id_member(original, "declaration", &row.declaration, at)?;
        }
    } else if root.contains_key("temporal_bindings") {
        return Err(malformed(JsonPointer::root().key("temporal_bindings")));
    }
    if let Some(rows) = &wire.schedulability_claims {
        let originals = raw_array(
            root.get("schedulability_claims").copied(),
            &["schedulability_claims"],
        )?;
        if rows.is_empty() || originals.len() != rows.len() {
            return Err(malformed(JsonPointer::root().key("schedulability_claims")));
        }
        for (index, (original, row)) in originals.iter().zip(rows).enumerate() {
            let at = JsonPointer::root()
                .key("schedulability_claims")
                .index(index);
            super::lossless::compare(original, row, at.clone(), &["declaration"])?;
            compare_id_member(original, "declaration", &row.declaration, at)?;
        }
    } else if root.contains_key("schedulability_claims") {
        return Err(malformed(JsonPointer::root().key("schedulability_claims")));
    }
    compare_items(
        &["diagnostics", "entries"],
        &diagnostic_entries,
        &wire.diagnostics.entries,
        &["details"],
    )
}

fn compare_id(
    raw: &RawValue,
    typed: &CheckedNodeId,
    at: JsonPointer,
) -> Result<(), ValidationFailure> {
    let decoded: CheckedNodeId =
        serde_json::from_str(raw.get()).map_err(|_| malformed(at.clone()))?;
    if decoded == *typed {
        Ok(())
    } else {
        Err(malformed(at))
    }
}

fn compare_id_member(
    raw: &RawValue,
    key: &str,
    typed: &CheckedNodeId,
    parent: JsonPointer,
) -> Result<(), ValidationFailure> {
    let at = parent.key(key);
    let value = raw_member(Some(raw), key).ok_or_else(|| malformed(at.clone()))?;
    compare_id(value, typed, at)
}

fn compare_root<T: serde::Serialize>(
    root: &BTreeMap<String, &RawValue>,
    key: &str,
    typed: &T,
    skipped_members: &'static [&'static str],
) -> Result<(), ValidationFailure> {
    let at = JsonPointer::root().key(key);
    let raw = root.get(key).ok_or_else(|| malformed(at.clone()))?;
    super::lossless::compare(raw, typed, at, skipped_members)
}

fn compare_items<T: serde::Serialize>(
    path: &[&str],
    originals: &[&RawValue],
    typed: &[T],
    skipped_members: &'static [&'static str],
) -> Result<(), ValidationFailure> {
    let pointer = path
        .iter()
        .fold(JsonPointer::root(), |pointer, key| pointer.key(key));
    if originals.len() != typed.len() {
        return Err(malformed(pointer));
    }
    for (index, (original, typed)) in originals.iter().zip(typed).enumerate() {
        super::lossless::compare(
            original,
            typed,
            pointer.clone().index(index),
            skipped_members,
        )?;
    }
    Ok(())
}

fn raw_member<'a>(parent: Option<&'a RawValue>, key: &str) -> Option<&'a RawValue> {
    let members: BTreeMap<String, &'a RawValue> = serde_json::from_str(parent?.get()).ok()?;
    members.get(key).copied()
}

fn raw_array<'a>(
    raw: Option<&'a RawValue>,
    path: &[&str],
) -> Result<Vec<&'a RawValue>, ValidationFailure> {
    let pointer = path
        .iter()
        .fold(JsonPointer::root(), |pointer, key| pointer.key(key));
    serde_json::from_str(raw.ok_or_else(|| malformed(pointer.clone()))?.get())
        .map_err(|_| malformed(pointer))
}

#[cfg(test)]
mod ordinal_locus_tests {
    use super::*;

    #[test]
    fn source_lookup_skips_deep_sibling_without_deserializing_it() {
        let deep = format!("{}0{}", "[".repeat(120), "]".repeat(120));
        let bytes = format!(
            "{{\"semantic_graph\":{{\"nodes\":[{{\"body\":{deep},\"node_id\":{{\"package\":\"p\",\"path\":[\"x\"],\"ordinal\":0}}}}]}}}}"
        );
        let pointer = JsonPointer::parse("/semantic_graph/nodes/0/node_id/ordinal").unwrap();
        assert_eq!(
            source_value(bytes.as_bytes(), &pointer),
            Some(Value::from(0))
        );
    }

    #[test]
    fn source_lookup_handles_escaped_member_and_string_contents() {
        let bytes = br#"{"a/b":{"~name":["quoted \" { [ }",{"node":3}]}}"#;
        let pointer = JsonPointer::parse("/a~1b/~0name/1/node").unwrap();
        assert_eq!(source_value(bytes, &pointer), Some(Value::from(3)));
    }

    #[test]
    fn malformed_node_id_members_keep_their_exact_pointer() {
        let node = JsonPointer::parse("/semantic_graph/nodes/1/node_id").unwrap();
        for ordinal in ["9007199254740992", "1.5", "-1", "\"one\""] {
            let bytes = format!(
                "{{\"semantic_graph\":{{\"nodes\":[null,{{\"node_id\":{{\"package\":\"p\",\"path\":[\"x\"],\"ordinal\":{ordinal}}}}}]}}}}"
            );
            assert_eq!(
                invalid_node_id_field(bytes.as_bytes(), &node),
                Some(node.clone().key("ordinal"))
            );
        }

        let nested = br#"{"semantic_graph":{"nodes":[null,{"node_id":{"package":"p","path":[{"package":"p","path":["x"],"ordinal":1.5}],"ordinal":0}}]}}"#;
        assert_eq!(
            invalid_node_id_field(nested, &node),
            Some(node.key("path").index(0))
        );

        let dependency = br#"{"semantic_graph":{"nodes":[{"dependencies":[{"package":"p","path":["x"],"ordinal":"one"}]}]}}"#;
        let reference = JsonPointer::parse("/semantic_graph/nodes/0/dependencies/0").unwrap();
        assert_eq!(
            invalid_node_id_field(dependency, &reference),
            Some(reference.key("ordinal"))
        );
    }
}
