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
    /// The handle for `node` of `document`. A later stage of the read builds
    /// each handle from the package document it retains; until then the read
    /// builds a fresh document per value through [`Self::from_value`].
    pub(in crate::checked_package) fn new(document: Arc<Document>, node: NodeId) -> Self {
        Self { document, node }
    }

    /// Reads one JSON value with the shared reader and refuses bytes that are
    /// not its canonical encoding. This constructor charges no budget. The
    /// read path currently reaches it through [`Self::from_value`] for each
    /// diagnostic detail, with an uncharged document per value.
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
        self.node().encode_into(writer)
    }
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
        let value = Value::deserialize(deserializer)?;
        Self::from_value(&value).map_err(serde::de::Error::custom)
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
    use serde_json::json;

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
