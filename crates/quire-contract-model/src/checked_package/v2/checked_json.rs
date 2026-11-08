// SPDX-License-Identifier: AGPL-3.0-or-later
//! `CheckedJson`: an owned handle to one value of a shared
//! `quire_canonical::Document`.
//!
//! The strict read builds every handle from the one package document it
//! retains, so a literal's value and a diagnostic's details are not copied
//! into a tree. Implements: FR-038 "The `CheckedJson` payload type".

use quire_canonical::{
    Document, Encode, Error as CanonicalError, Limits, Node, NodeId, NodeRef, ReadError, Sink,
    Writer,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

/// The name under which the read's document deserializer hands out a handle
/// instead of a copied value.
pub(super) const CHECKED_JSON_TOKEN: &str = "$quire::CheckedJson";

thread_local! {
    static OFFERED: std::cell::RefCell<Option<CheckedJson>> =
        const { std::cell::RefCell::new(None) };
}

/// Why a `CheckedJson` could not be built outside the read.
#[derive(Debug)]
pub enum CheckedJsonError {
    /// The shared reader refused the bytes.
    Read(ReadError),
    /// The canonical encoder refused the value.
    Encode(CanonicalError),
    /// The bytes are valid JSON but not their own canonical encoding.
    NotCanonical,
}

impl fmt::Display for CheckedJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(formatter, "{error}"),
            Self::Encode(error) => write!(formatter, "{error}"),
            Self::NotCanonical => formatter.write_str("the bytes are not canonical JSON"),
        }
    }
}

impl std::error::Error for CheckedJsonError {}

/// A cheap-to-clone handle to one JSON value of a shared document.
///
/// Two handles are equal when their values have equal canonical encodings,
/// whatever document or node they name.
#[derive(Clone)]
pub struct CheckedJson {
    document: Arc<Document>,
    node: NodeId,
}

impl CheckedJson {
    /// The document this handle names a value of.
    pub(in crate::checked_package) fn document(&self) -> &Arc<Document> {
        &self.document
    }

    /// The handle for `node` of `document`. Used by the read, which builds
    /// each handle from the package document it retains.
    pub(in crate::checked_package) fn new(document: Arc<Document>, node: NodeId) -> Self {
        Self { document, node }
    }

    /// A handle for a scalar literal payload, read as written: a number the
    /// canonical encoder cannot write (an integer past 2^53) is held and is
    /// the encoder's refusal when the package is written, as it always was.
    pub(in crate::checked_package) fn scalar(value: &Value) -> Self {
        let read = |text: &[u8]| quire_canonical::read(text, u64::MAX).ok();
        // A scalar's JSON text always reads; `null` is the fallback and is
        // valid JSON, so the final read cannot fail.
        let document = read(value.to_string().as_bytes())
            .or_else(|| read(b"null"))
            .expect("`null` is valid JSON");
        let node = document.root().id();
        Self::new(Arc::new(document), node)
    }

    /// Leaves `handle` for the `Deserialize` call the document deserializer
    /// is answering on this thread.
    pub(super) fn offer(handle: Self) {
        OFFERED.with(|slot| *slot.borrow_mut() = Some(handle));
    }

    /// Reads one JSON value with the shared reader and refuses bytes that are
    /// not its canonical encoding. Neither this constructor nor
    /// [`Self::from_value`] is on the read path or charges a budget.
    ///
    /// # Errors
    ///
    /// [`CheckedJsonError::Read`] when the reader refuses the bytes, and
    /// [`CheckedJsonError::NotCanonical`] when they are not canonical.
    pub fn from_canonical_bytes(
        bytes: &[u8],
        max_input_bytes: u64,
    ) -> Result<Self, CheckedJsonError> {
        let document =
            quire_canonical::read(bytes, max_input_bytes).map_err(CheckedJsonError::Read)?;
        let canonical = quire_canonical::to_vec(&document, Limits::new(u64::MAX))
            .map_err(CheckedJsonError::Encode)?;
        if canonical != bytes {
            return Err(CheckedJsonError::NotCanonical);
        }
        let node = document.root().id();
        Ok(Self::new(Arc::new(document), node))
    }

    /// Encodes `value` canonically and reads it back.
    ///
    /// # Errors
    ///
    /// [`CheckedJsonError::Encode`] when `value` has no canonical encoding,
    /// and [`CheckedJsonError::Read`] when the reader refuses its encoding.
    pub fn from_value(value: &Value) -> Result<Self, CheckedJsonError> {
        let bytes = quire_canonical::to_vec(value, Limits::new(u64::MAX))
            .map_err(CheckedJsonError::Encode)?;
        Self::from_canonical_bytes(&bytes, u64::MAX)
    }

    /// The value, borrowed through the shared reader's node API.
    #[must_use]
    pub fn node(&self) -> NodeRef<'_> {
        // The id was produced by this document; the fallback keeps a handle
        // total without a panic path.
        self.document
            .node(self.node)
            .unwrap_or_else(|| self.document.root())
    }

    /// The string the value is, borrowed from the document.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self.node().node() {
            Node::String(text) => Some(text),
            _ => None,
        }
    }

    /// The boolean the value is.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self.node().node() {
            Node::Bool(flag) => Some(flag),
            _ => None,
        }
    }

    /// The unsigned integer the value spells, as `serde_json` reads one.
    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        match self.node().node() {
            Node::Number(number) => number.text().parse().ok(),
            _ => None,
        }
    }

    /// The signed integer the value spells, as `serde_json` reads one.
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self.node().node() {
            Node::Number(number) => number.text().parse().ok(),
            _ => None,
        }
    }

    /// The source text of the number the value is.
    #[must_use]
    pub fn number_text(&self) -> Option<&str> {
        match self.node().node() {
            Node::Number(number) => Some(number.text()),
            _ => None,
        }
    }

    /// Whether the value is `null`.
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self.node().node(), Node::Null)
    }

    /// Whether the value is a string.
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self.node().node(), Node::String(_))
    }

    /// Whether the value is an array.
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self.node().node(), Node::Array(_))
    }

    /// Whether the value is an object.
    #[must_use]
    pub fn is_object(&self) -> bool {
        matches!(self.node().node(), Node::Object(_))
    }

    /// Copies the value into a `serde_json::Value`. Not on the read path.
    #[must_use]
    pub fn to_value(&self) -> Value {
        value_of(self.node())
    }

    /// The canonical bytes of the value, encoded from an explicit heap stack.
    fn canonical_bytes(&self) -> Option<Vec<u8>> {
        quire_canonical::to_vec(self, Limits::new(u64::MAX)).ok()
    }
}

impl Encode for CheckedJson {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        encode_node(self.node(), writer)
    }
}

/// Writes one document value from an explicit heap stack, kept on the
/// writer's reservation. An integer past 2^53 is the encoder's refusal, as it
/// is for a `serde_json` number: the document holds the number's text, so a
/// double would otherwise silently round it.
pub(super) fn encode_node<S: Sink + ?Sized>(
    root: NodeRef<'_>,
    writer: &mut Writer<'_, S>,
) -> Result<(), CanonicalError> {
    const MAXIMUM_INTEGER: u128 = 1 << 53;
    enum Task<'d> {
        Value(NodeRef<'d>),
        Name(&'d str),
        EndArray,
        EndObject,
    }
    let mut tasks = Vec::new();
    writer.reserve_list(&mut tasks, 1)?;
    tasks.push(Task::Value(root));
    while let Some(task) = tasks.pop() {
        match task {
            Task::Value(value) => match value.node() {
                Node::Null => writer.null()?,
                Node::Bool(flag) => writer.bool(flag)?,
                Node::Number(number) => match number.text().parse::<i128>() {
                    Ok(integer) if integer.unsigned_abs() > MAXIMUM_INTEGER => {
                        return Err(CanonicalError::IntegerMagnitudeAboveMaximum(integer));
                    }
                    Ok(integer) => writer.integer(integer)?,
                    Err(_) => {
                        let float = number.value();
                        if float.fract() == 0.0 && float.abs() > MAXIMUM_INTEGER as f64 {
                            return Err(CanonicalError::Internal {
                                invariant: "an integer past 2^53 is not canonical",
                            });
                        }
                        writer.number(float)?;
                    }
                },
                Node::String(text) => writer.string(text)?,
                Node::Array(items) => {
                    writer.begin_array()?;
                    let start = tasks.len();
                    writer.reserve_list(&mut tasks, items.len().saturating_add(1))?;
                    tasks.push(Task::EndArray);
                    tasks.extend(items.map(Task::Value));
                    tasks.get_mut(start + 1..).unwrap_or_default().reverse();
                }
                Node::Object(members) => {
                    writer.begin_object()?;
                    let start = tasks.len();
                    writer.reserve_list(
                        &mut tasks,
                        members.len().saturating_mul(2).saturating_add(1),
                    )?;
                    tasks.push(Task::EndObject);
                    for (name, member) in members {
                        tasks.push(Task::Name(name));
                        tasks.push(Task::Value(member));
                    }
                    tasks.get_mut(start + 1..).unwrap_or_default().reverse();
                }
            },
            Task::Name(name) => writer.name(name)?,
            Task::EndArray => writer.end_array()?,
            Task::EndObject => writer.end_object()?,
        }
    }
    Ok(())
}

impl PartialEq for CheckedJson {
    fn eq(&self, other: &Self) -> bool {
        match (self.canonical_bytes(), other.canonical_bytes()) {
            (Some(left), Some(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for CheckedJson {}

impl fmt::Debug for CheckedJson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.canonical_bytes() {
            Some(bytes) => formatter.write_str(&String::from_utf8_lossy(&bytes)),
            None => formatter.write_str("<unencodable>"),
        }
    }
}

impl Serialize for CheckedJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_value().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CheckedJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Held;
        impl<'de> serde::de::Visitor<'de> for Held {
            type Value = CheckedJson;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON value")
            }

            // The document deserializer answers with the handle it offered.
            fn visit_unit<E: serde::de::Error>(self) -> Result<CheckedJson, E> {
                OFFERED
                    .with(|slot| slot.borrow_mut().take())
                    .ok_or_else(|| E::custom("no JSON handle was offered"))
            }

            // Any other deserializer yields a value to encode.
            fn visit_newtype_struct<D2: Deserializer<'de>>(
                self,
                inner: D2,
            ) -> Result<CheckedJson, D2::Error> {
                let value = Value::deserialize(inner)?;
                CheckedJson::from_value(&value).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_newtype_struct(CHECKED_JSON_TOKEN, Held)
    }
}

/// The `Value` of one document node, built from an explicit heap stack so a
/// value of any depth converts on any thread stack.
fn value_of(root: NodeRef<'_>) -> Value {
    use quire_canonical::{Items, Members};
    enum Open<'d> {
        Array(Vec<Value>, Items<'d>),
        Object(serde_json::Map<String, Value>, Members<'d>, String),
    }
    let mut open: Vec<Open<'_>> = Vec::new();
    let mut pending = Some(root);
    loop {
        let mut finished = None;
        if let Some(node) = pending.take() {
            match node.node() {
                Node::Null => finished = Some(Value::Null),
                Node::Bool(flag) => finished = Some(Value::Bool(flag)),
                Node::Number(number) => finished = Some(number_value(number)),
                Node::String(text) => finished = Some(Value::String(text.to_owned())),
                Node::Array(items) => open.push(Open::Array(Vec::new(), items)),
                Node::Object(members) => {
                    open.push(Open::Object(serde_json::Map::new(), members, String::new()));
                }
            }
        }
        loop {
            if let Some(value) = finished.take() {
                match open.last_mut() {
                    None => return value,
                    Some(Open::Array(elements, _)) => elements.push(value),
                    Some(Open::Object(members, _, name)) => {
                        members.insert(std::mem::take(name), value);
                    }
                }
            }
            match open.last_mut() {
                None => return Value::Null,
                Some(Open::Array(_, items)) => match items.next() {
                    Some(item) => {
                        pending = Some(item);
                        break;
                    }
                    None => {
                        if let Some(Open::Array(elements, _)) = open.pop() {
                            finished = Some(Value::Array(elements));
                        }
                    }
                },
                Some(Open::Object(_, members, name)) => match members.next() {
                    Some((key, member)) => {
                        key.clone_into(name);
                        pending = Some(member);
                        break;
                    }
                    None => {
                        if let Some(Open::Object(members, _, _)) = open.pop() {
                            finished = Some(Value::Object(members));
                        }
                    }
                },
            }
        }
    }
}

/// The integer a number's text spells when it fits an `i64` or `u64`, and the
/// double it denotes otherwise.
fn number_value(number: quire_canonical::Number<'_>) -> Value {
    let text = number.text();
    if let Ok(integer) = text.parse::<i64>() {
        return Value::from(integer);
    }
    if let Ok(integer) = text.parse::<u64>() {
        return Value::from(integer);
    }
    serde_json::Number::from_f64(number.value()).map_or(Value::Null, Value::Number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-472", "FR-038-AC-514")]
    #[test]
    fn handles_round_trip_compare_by_canonical_encoding_and_refuse_noncanonical_bytes() {
        let value = json!({"b": [1, 2.5, "x", null, true], "a": {"k": -7}});
        let left = CheckedJson::from_value(&value).expect("encodes");
        assert_eq!(left.to_value(), value);
        assert_eq!(
            format!("{left:?}"),
            r#"{"a":{"k":-7},"b":[1,2.5,"x",null,true]}"#
        );
        let right = CheckedJson::from_canonical_bytes(
            br#"{"a":{"k":-7},"b":[1,2.5,"x",null,true]}"#,
            u64::MAX,
        )
        .expect("canonical");
        assert_eq!(left, right);
        assert_eq!(left.clone(), left);
        assert_ne!(left, CheckedJson::from_value(&json!([1])).expect("encodes"));
        assert!(matches!(
            CheckedJson::from_canonical_bytes(br#"{"b":1,"a":2}"#, u64::MAX),
            Err(CheckedJsonError::NotCanonical)
        ));
        let text = serde_json::to_string(&left).expect("serializes");
        let back: CheckedJson = serde_json::from_str(&text).expect("deserializes");
        assert_eq!(back, left);
        assert!(matches!(left.node().node(), Node::Object(_)));
    }

    #[trace("TC-472", "FR-038-AC-514")]
    #[test]
    fn a_deep_value_compares_and_drops_without_native_recursion() {
        let depth = 100_000;
        let text = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
        let handle = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || {
                let one =
                    CheckedJson::from_canonical_bytes(text.as_bytes(), u64::MAX).expect("read");
                let two = one.clone();
                assert_eq!(one, two);
                drop(one);
                drop(two);
            })
            .expect("spawn");
        handle.join().expect("no stack overflow");
    }
}
