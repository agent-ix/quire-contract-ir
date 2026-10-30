//! The closed-schema decode of the package document, without native recursion
//! following the nesting of a term or structural node path.
//!
//! A node `body` and a diagnostic's `details`
//! entry may nest as deep as the strict parser's limit of 128. The outer typed
//! decode borrows each such term as a raw source slice, then decodes it into a
//! `Value` only after that outer frame has returned. A `NodeId.path` is
//! captured and decoded on a heap worklist, so it has no fixed depth cap.
//! This keeps an adverse deep term or path within the reader stack budget.
//!
//! The lossless-decode check (no member defaulted, nulled or dropped) compares
//! borrowed source array items against their typed serialization one at a
//! time, except structural node identities, which compare from raw slices
//! to avoid recursive serialization.

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedNode<'a> {
    node_id: CheckedNodeId,
    schema_version: Box<str>,
    node_tag: Box<str>,
    semantic_form: Box<str>,
    semantic_type: CheckedNodeId,
    dependencies: Vec<CheckedNodeId>,
    occurrences: Vec<CheckedOccurrence>,
    #[serde(default)]
    recursion_group: Option<Box<str>>,
    #[serde(default, deserialize_with = "present_nullable")]
    declaration: Option<Option<CheckedDeclaration>>,
    #[serde(default)]
    owner: Option<CheckedNodeOwner>,
    #[serde(default)]
    owner_model: Option<CheckedNodeId>,
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
            if refusal.code == CheckedPackageRefusalCode::MalformedWire {
                if let Some(at) = refusal.primary_path().cloned() {
                    if let Some(ordinal) = invalid_node_id_ordinal(bytes, &at) {
                        refusal.set_primary_path(Some(ordinal));
                    }
                }
            }
        }
        failure
    })?;
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
            node_id: node.node_id,
            schema_version: node.schema_version,
            node_tag: node.node_tag,
            semantic_form: node.semantic_form,
            semantic_type: node.semantic_type,
            dependencies: node.dependencies,
            occurrences: node.occurrences,
            recursion_group: node.recursion_group,
            declaration: node.declaration.flatten(),
            owner: node.owner,
            owner_model: node.owner_model,
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

/// Refine a buffered structural ID decode error to the actual ordinal member.
/// Nested operand IDs are walked on the heap, without a path-depth cap.
fn invalid_node_id_ordinal(bytes: &[u8], at: &JsonPointer) -> Option<JsonPointer> {
    const MAX_SAFE: u64 = 9_007_199_254_740_991;
    let mut pending = vec![(source_raw(bytes, at)?, at.clone())];
    while let Some((raw, path)) = pending.pop() {
        let members: BTreeMap<&str, &RawValue> = serde_json::from_str(raw.get()).ok()?;
        if let Some(ordinal) = members.get("ordinal") {
            if serde_json::from_str::<u64>(ordinal.get())
                .ok()
                .is_none_or(|value| value > MAX_SAFE)
            {
                return Some(path.key("ordinal"));
            }
        }
        if let Some(raw_path) = members.get("path") {
            let segments: Vec<&RawValue> = serde_json::from_str(raw_path.get()).ok()?;
            for (index, segment) in segments.into_iter().enumerate().rev() {
                if segment.get().starts_with('{') {
                    pending.push((segment, path.clone().key("path").index(index)));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod ordinal_locus_tests {
    use super::*;

    #[test]
    fn malformed_structural_ordinals_keep_their_exact_member_pointer() {
        let node = JsonPointer::parse("/semantic_graph/nodes/1/node_id").unwrap();
        for ordinal in ["9007199254740992", "1.5", "-1", "\"one\""] {
            let bytes = format!(
                "{{\"semantic_graph\":{{\"nodes\":[null,{{\"node_id\":{{\"package\":\"p\",\"path\":[\"x\"],\"ordinal\":{ordinal}}}}}]}}}}"
            );
            assert_eq!(
                invalid_node_id_ordinal(bytes.as_bytes(), &node),
                Some(node.clone().key("ordinal"))
            );
        }

        let nested = br#"{"semantic_graph":{"nodes":[null,{"node_id":{"package":"p","path":[{"package":"p","path":["x"],"ordinal":1.5}],"ordinal":0}}]}}"#;
        assert_eq!(
            invalid_node_id_ordinal(nested, &node),
            Some(node.key("path").index(0).key("ordinal"))
        );

        let dependency = br#"{"semantic_graph":{"nodes":[{"dependencies":[{"package":"p","path":["x"],"ordinal":"one"}]}]}}"#;
        let reference = JsonPointer::parse("/semantic_graph/nodes/0/dependencies/0").unwrap();
        assert_eq!(
            invalid_node_id_ordinal(dependency, &reference),
            Some(reference.key("ordinal"))
        );
    }
}

/// Read only the JSON value named by `pointer`. Borrowed raw subtrees retain
/// slices of the caller's bytes; no ancestor is decoded into a `Value`.
pub(super) fn source_value(bytes: &[u8], pointer: &JsonPointer) -> Option<Value> {
    serde_json::from_str(source_raw(bytes, pointer)?.get()).ok()
}

fn source_raw<'a>(bytes: &'a [u8], pointer: &JsonPointer) -> Option<&'a RawValue> {
    let mut current: &RawValue = serde_json::from_slice(bytes).ok()?;
    let path = pointer.as_str();
    if !path.is_empty() {
        for token in path.strip_prefix('/')?.split('/') {
            let key = token.replace("~1", "/").replace("~0", "~");
            current = match current.get().as_bytes().first()? {
                b'{' => {
                    let members: BTreeMap<String, &RawValue> =
                        serde_json::from_str(current.get()).ok()?;
                    *members.get(&key)?
                }
                b'[' => {
                    let items: Vec<&RawValue> = serde_json::from_str(current.get()).ok()?;
                    *items.get(key.parse::<usize>().ok()?)?
                }
                _ => return None,
            };
        }
    }
    Some(current)
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
