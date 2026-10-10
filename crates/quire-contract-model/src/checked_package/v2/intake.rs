//! The closed-schema decode of the package document, without a recursion that
//! follows the nesting of a term or a copy of the document that follows its size.
//!
//! A node `body`, an identity-projection `body` and a diagnostic's `details`
//! entry are `serde_json::Value`s of any shape the strict parse reads (up to its
//! recursion limit of 128), and `serde` re-reads a `Value` into a `Value` once
//! per level on the call stack, several frames each: a body nested deeper than
//! the closed body grammar allows must refuse `malformed_wire` at the body
//! grammar, never overflow the stack (FR-038-AC-117). So these values are taken
//! out of the document before the decode, which then reads only the closed
//! members around them, and put into the decoded wire after it; the bodies the
//! decode never touches are checked by the iterative body-grammar walk.
//!
//! The lossless-decode check (no member defaulted, nulled or dropped) compares
//! the document with the wire's serialization. It takes the three long arrays
//! (the nodes, the identity projection and the source map) out of both and
//! compares them element by element, so no second copy of the whole document is
//! built.

use super::{CheckedPackageWireV2, ValidationFailure};
use crate::checked_package::common::first_difference;
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
use serde::Serialize;
use serde_json::value::RawValue;
use serde_json::Value;
use std::collections::BTreeMap;

/// Read only the JSON value named by `pointer`. Borrowed raw subtrees retain
/// slices of the caller's bytes; no ancestor is decoded into a `Value`.
pub(super) fn source_value(bytes: &[u8], pointer: &JsonPointer) -> Option<Value> {
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
    serde_json::from_str(current.get()).ok()
}

/// The array at `path` below `value`, or an empty slice when there is none.
fn items_mut<'a>(value: &'a mut Value, path: &[&str]) -> &'a mut [Value] {
    let mut at = value;
    for key in path {
        match at.get_mut(*key) {
            Some(next) => at = next,
            None => return &mut [],
        }
    }
    match at {
        Value::Array(items) => items.as_mut_slice(),
        _ => &mut [],
    }
}

/// Takes the `body` of every node and of every identity-projection entry and
/// every `details` entry out of `document`, leaving `null` in its place: the
/// nodes' bodies first, then the projection's, then the details, each in
/// document order. [`attach_terms`] puts them back in the same order.
pub(super) fn detach_terms(document: &mut Value) -> Vec<Value> {
    let mut terms = Vec::new();
    for path in [
        ["semantic_graph", "nodes"],
        ["identity_preimage", "identity_projection"],
    ] {
        for node in items_mut(document, &path) {
            terms.push(node.get_mut("body").map(std::mem::take).unwrap_or_default());
        }
    }
    for entry in items_mut(document, &["diagnostics", "entries"]) {
        if let Some(Value::Array(details)) = entry.get_mut("details") {
            terms.extend(details.iter_mut().map(std::mem::take));
        }
    }
    terms
}

/// Puts the values [`detach_terms`] took back into the decoded `wire`. The
/// decode read one `null` in the place of each, so the wire holds exactly one
/// slot for each value taken, in the same order.
pub(super) fn attach_terms(wire: &mut CheckedPackageWireV2, terms: Vec<Value>) {
    let mut terms = terms.into_iter();
    let mut next = || terms.next().unwrap_or_default();
    for node in &mut wire.semantic_graph.nodes {
        node.body = next();
    }
    for node in &mut wire.identity_preimage.identity_projection {
        node.body = next();
    }
    for entry in &mut wire.diagnostics.entries {
        for detail in &mut entry.details {
            *detail = next();
        }
    }
}

/// The array at `path` below `value`, taken out and leaving an empty one;
/// `None` when there is none.
fn take_array(value: &mut Value, path: &[&str]) -> Option<Vec<Value>> {
    let mut at = value;
    for key in path {
        at = at.get_mut(*key)?;
    }
    match at {
        Value::Array(items) => Some(std::mem::take(items)),
        _ => None,
    }
}

/// `malformed_wire` at `at`.
fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

/// Compares the elements of the array `original` at `at` with the serialization
/// of each of `typed`, one at a time: `malformed_wire` at the array when its
/// length differs, else at the first member, in document order, at which an
/// element differs.
fn compare_items<T: Serialize>(
    at: &[&str],
    original: Option<Vec<Value>>,
    typed: &[T],
) -> Result<(), ValidationFailure> {
    let pointer = at
        .iter()
        .fold(JsonPointer::root(), |pointer, key| pointer.key(key));
    let Some(original) = original.filter(|items| items.len() == typed.len()) else {
        return Err(malformed(pointer));
    };
    for (index, (original, typed)) in original.iter().zip(typed).enumerate() {
        let decoded = serde_json::to_value(typed).map_err(|_| malformed(JsonPointer::root()))?;
        if decoded != *original {
            return Err(malformed(first_difference(
                pointer.clone().index(index),
                original,
                &decoded,
            )));
        }
    }
    Ok(())
}

/// A lossless decode: no member of `document` was defaulted, nulled or dropped
/// by the decode into `wire`. Both hold `null` for the values
/// [`detach_terms`] took; the nodes, the identity projection and the source map
/// are compared one element at a time, the rest as a whole.
pub(super) fn check_lossless(
    document: &mut Value,
    wire: &mut CheckedPackageWireV2,
) -> Result<(), ValidationFailure> {
    let nodes = take_array(document, &["semantic_graph", "nodes"]);
    let projection = take_array(document, &["identity_preimage", "identity_projection"]);
    let source_map = take_array(document, &["source_map"]);
    let typed_nodes = std::mem::take(&mut wire.semantic_graph.nodes);
    let typed_projection = std::mem::take(&mut wire.identity_preimage.identity_projection);
    let typed_source_map = std::mem::take(&mut wire.source_map);
    let outcome = (|| {
        let rest = serde_json::to_value(&*wire).map_err(|_| malformed(JsonPointer::root()))?;
        if rest != *document {
            return Err(malformed(first_difference(
                JsonPointer::root(),
                document,
                &rest,
            )));
        }
        compare_items(
            &["identity_preimage", "identity_projection"],
            projection,
            &typed_projection,
        )?;
        compare_items(&["semantic_graph", "nodes"], nodes, &typed_nodes)?;
        compare_items(&["source_map"], source_map, &typed_source_map)
    })();
    wire.semantic_graph.nodes = typed_nodes;
    wire.identity_preimage.identity_projection = typed_projection;
    wire.source_map = typed_source_map;
    outcome
}

/// Check the lossless closed decode against borrowed slices of the source.
/// Large arrays are decoded one item at a time, so this check never owns a
/// second document tree. `body` values already came directly from the same
/// source and are held aside while the surrounding closed members are checked.
pub(super) fn check_lossless_source(
    bytes: &[u8],
    wire: &mut CheckedPackageWireV2,
) -> Result<(), ValidationFailure> {
    let root: BTreeMap<String, &RawValue> =
        serde_json::from_slice(bytes).map_err(|_| malformed(JsonPointer::root()))?;
    let nodes = raw_array(
        raw_member(root.get("semantic_graph").copied(), "nodes"),
        &["semantic_graph", "nodes"],
    )?;
    let projection = raw_array(
        raw_member(
            root.get("identity_preimage").copied(),
            "identity_projection",
        ),
        &["identity_preimage", "identity_projection"],
    )?;
    let source_map = raw_array(root.get("source_map").copied(), &["source_map"])?;

    let mut original = serde_json::Map::new();
    for (key, raw) in &root {
        let value = match key.as_str() {
            "semantic_graph" => reduced_object(raw, "nodes")?,
            "identity_preimage" => reduced_object(raw, "identity_projection")?,
            "source_map" => Value::Array(Vec::new()),
            _ => serde_json::from_str(raw.get()).map_err(|_| malformed(JsonPointer::root()))?,
        };
        original.insert(key.clone(), value);
    }

    let mut typed_nodes = std::mem::take(&mut wire.semantic_graph.nodes);
    let mut typed_projection = std::mem::take(&mut wire.identity_preimage.identity_projection);
    let typed_source_map = std::mem::take(&mut wire.source_map);
    let outcome = (|| {
        let rest = serde_json::to_value(&*wire).map_err(|_| malformed(JsonPointer::root()))?;
        let original = Value::Object(original);
        if rest != original {
            return Err(malformed(first_difference(
                JsonPointer::root(),
                &original,
                &rest,
            )));
        }
        compare_bodies(
            &["identity_preimage", "identity_projection"],
            &projection,
            &mut typed_projection,
        )?;
        compare_bodies(&["semantic_graph", "nodes"], &nodes, &mut typed_nodes)?;
        compare_raw_items(&["source_map"], &source_map, &typed_source_map)
    })();
    wire.semantic_graph.nodes = typed_nodes;
    wire.identity_preimage.identity_projection = typed_projection;
    wire.source_map = typed_source_map;
    outcome
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

fn reduced_object(raw: &RawValue, omitted: &str) -> Result<Value, ValidationFailure> {
    let members: BTreeMap<String, &RawValue> =
        serde_json::from_str(raw.get()).map_err(|_| malformed(JsonPointer::root()))?;
    let mut value = serde_json::Map::new();
    for (key, raw) in members {
        value.insert(
            key.clone(),
            if key == omitted {
                Value::Array(Vec::new())
            } else {
                serde_json::from_str(raw.get()).map_err(|_| malformed(JsonPointer::root()))?
            },
        );
    }
    Ok(Value::Object(value))
}

fn compare_raw_items<T: Serialize>(
    path: &[&str],
    originals: &[&RawValue],
    typed: &[T],
) -> Result<(), ValidationFailure> {
    let pointer = path
        .iter()
        .fold(JsonPointer::root(), |pointer, key| pointer.key(key));
    if originals.len() != typed.len() {
        return Err(malformed(pointer));
    }
    for (index, (original, typed)) in originals.iter().zip(typed).enumerate() {
        let original: Value = serde_json::from_str(original.get())
            .map_err(|_| malformed(pointer.clone().index(index)))?;
        let decoded =
            serde_json::to_value(typed).map_err(|_| malformed(pointer.clone().index(index)))?;
        if original != decoded {
            return Err(malformed(first_difference(
                pointer.clone().index(index),
                &original,
                &decoded,
            )));
        }
    }
    Ok(())
}

trait BodyHolder: Serialize {
    fn body_mut(&mut self) -> &mut Value;
}

impl BodyHolder for super::CheckedSemanticNodeV2 {
    fn body_mut(&mut self) -> &mut Value {
        &mut self.body
    }
}

impl BodyHolder for super::CheckedNodeProjectionV2 {
    fn body_mut(&mut self) -> &mut Value {
        &mut self.body
    }
}

fn compare_bodies<T: BodyHolder>(
    path: &[&str],
    originals: &[&RawValue],
    typed: &mut [T],
) -> Result<(), ValidationFailure> {
    let pointer = path
        .iter()
        .fold(JsonPointer::root(), |pointer, key| pointer.key(key));
    if originals.len() != typed.len() {
        return Err(malformed(pointer));
    }
    for (index, (original, typed)) in originals.iter().zip(typed).enumerate() {
        let at = pointer.clone().index(index);
        let mut original: Value =
            serde_json::from_str(original.get()).map_err(|_| malformed(at.clone()))?;
        if let Some(object) = original.as_object_mut() {
            if object.contains_key("body") {
                object.insert("body".to_owned(), Value::Null);
            }
        }
        let body = std::mem::take(typed.body_mut());
        let decoded = serde_json::to_value(&*typed);
        *typed.body_mut() = body;
        let decoded = decoded.map_err(|_| malformed(at.clone()))?;
        if original != decoded {
            return Err(malformed(first_difference(at, &original, &decoded)));
        }
    }
    Ok(())
}
