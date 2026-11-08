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
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(test)]
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
    /// The value could not be serialized as JSON.
    Serialize(serde_json::Error),
    /// The bytes are valid JSON but not their own canonical encoding.
    NotCanonical,
}

impl fmt::Display for CheckedJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(formatter, "{error}"),
            Self::Encode(error) => write!(formatter, "{error}"),
            Self::Serialize(error) => write!(formatter, "{error}"),
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
    pub(in crate::checked_package) fn scalar(value: &impl Serialize) -> Self {
        let bytes = serde_json::to_vec(value).expect("a scalar literal serializes as JSON");
        let document =
            quire_canonical::read(&bytes, u64::MAX).expect("a scalar literal is a JSON value");
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
    pub fn from_serializable(value: &impl Serialize) -> Result<Self, CheckedJsonError> {
        let bytes = serde_json::to_vec(value).map_err(CheckedJsonError::Serialize)?;
        let document = quire_canonical::read(&bytes, u64::MAX).map_err(CheckedJsonError::Read)?;
        let node = document.root().id();
        let source = Self::new(Arc::new(document), node);
        let canonical = quire_canonical::to_vec(&source, Limits::new(u64::MAX))
            .map_err(CheckedJsonError::Encode)?;
        Self::from_canonical_bytes(&canonical, u64::MAX)
    }

    /// Test fixture constructor for an already materialized JSON value.
    #[cfg(test)]
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
    #[cfg(test)]
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

const SERDE_JSON_NUMBER_TOKEN: &str = "$serde_json::private::Number";

/// Writes one document value from an explicit heap stack, kept on the
/// writer's reservation. An integer past 2^53 is the encoder's refusal, as it
/// is for a `serde_json` number: the document holds the number's text, so a
/// double would otherwise silently round it.
// The one string comparison is a closed-vocabulary guard on a serde_json
// private member name, not an open key.
#[ix_trace_rs::string_edge]
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
                    // serde_json reads an object whose first member is this
                    // private name as a number, so two byte strings would
                    // decode to one package: the spelling is not canonical.
                    let mut members = members.peekable();
                    if members
                        .peek()
                        .is_some_and(|(name, _)| *name == SERDE_JSON_NUMBER_TOKEN)
                    {
                        return Err(CanonicalError::Internal {
                            invariant: "serde_json's number token is not canonical",
                        });
                    }
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
        serialize_node(self.node(), serializer)
    }
}

pub(super) fn serialize_node<S: Serializer>(
    node: NodeRef<'_>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match node.node() {
        Node::Null => serializer.serialize_unit(),
        Node::Bool(value) => serializer.serialize_bool(value),
        Node::Number(number) => {
            let text = number.text();
            if let Ok(value) = text.parse::<i64>() {
                serializer.serialize_i64(value)
            } else if let Ok(value) = text.parse::<u64>() {
                serializer.serialize_u64(value)
            } else {
                serializer.serialize_f64(number.value())
            }
        }
        Node::String(text) => serializer.serialize_str(text),
        Node::Array(items) => {
            let mut sequence = serializer.serialize_seq(None)?;
            for item in items {
                sequence.serialize_element(&NodeValue(item))?;
            }
            sequence.end()
        }
        Node::Object(members) => {
            let mut map = serializer.serialize_map(None)?;
            for (name, value) in members {
                map.serialize_entry(name, &NodeValue(value))?;
            }
            map.end()
        }
    }
}

struct NodeValue<'a>(NodeRef<'a>);

impl Serialize for NodeValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_node(self.0, serializer)
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
                let value = OwnedJson::deserialize(inner)?;
                CheckedJson::from_serializable(&value).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_newtype_struct(CHECKED_JSON_TOKEN, Held)
    }
}

enum OwnedJson {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    String(String),
    Array(Vec<OwnedJson>),
    Object(Vec<(String, OwnedJson)>),
}

impl Serialize for OwnedJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::Unsigned(value) => serializer.serialize_u64(*value),
            Self::Float(value) => serializer.serialize_f64(*value),
            Self::String(value) => serializer.serialize_str(value),
            Self::Array(values) => values.serialize(serializer),
            Self::Object(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for OwnedJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> serde::de::Visitor<'de> for JsonVisitor {
            type Value = OwnedJson;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON value")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(OwnedJson::Null)
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(OwnedJson::Null)
            }
            fn visit_some<D2: Deserializer<'de>>(
                self,
                deserializer: D2,
            ) -> Result<Self::Value, D2::Error> {
                OwnedJson::deserialize(deserializer)
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(OwnedJson::Bool(value))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(OwnedJson::Integer(value))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(OwnedJson::Unsigned(value))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(OwnedJson::Float(value))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(OwnedJson::String(value.to_owned()))
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(OwnedJson::String(value))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(OwnedJson::Array(values))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = map.next_entry()? {
                    values.push(value);
                }
                Ok(OwnedJson::Object(values))
            }
        }
        deserializer.deserialize_any(JsonVisitor)
    }
}

/// The `Value` of one document node, built from an explicit heap stack so a
/// value of any depth converts on any thread stack.
#[cfg(test)]
pub(in crate::checked_package) fn value_of(root: NodeRef<'_>) -> Value {
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
#[cfg(test)]
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

    #[trace("TC-048", "FR-038-AC-79", "FR-038-AC-514")]
    #[test]
    fn serializable_payload_refuses_large_integers_before_binary64_rounding() {
        for integer in [9_007_199_254_740_993_i64, -9_007_199_254_740_993_i64] {
            for value in [json!(integer), json!({"details": [integer]})] {
                assert!(matches!(
                    CheckedJson::from_serializable(&value),
                    Err(CheckedJsonError::Encode(
                        CanonicalError::IntegerMagnitudeAboveMaximum(actual)
                    )) if actual == i128::from(integer)
                ));
            }
        }
        assert!(matches!(
            CheckedJson::from_serializable(&9_007_199_254_740_993_u64),
            Err(CheckedJsonError::Encode(
                CanonicalError::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993)
            ))
        ));
    }

    #[trace("TC-048", "FR-038-AC-79", "FR-038-AC-514")]
    #[test]
    fn serializable_payload_preserves_integer_boundaries_and_fractional_values() {
        #[derive(Serialize)]
        struct Payload {
            integers: [i64; 3],
            ratio: f64,
        }
        let value = Payload {
            integers: [
                9_007_199_254_740_991,
                9_007_199_254_740_992,
                -9_007_199_254_740_992,
            ],
            ratio: 2.5,
        };
        let payload = CheckedJson::from_serializable(&value).expect("canonical payload");
        assert_eq!(
            payload.to_value(),
            serde_json::to_value(&value).expect("JSON value")
        );
        let canonical =
            quire_canonical::to_vec(&payload, Limits::new(u64::MAX)).expect("canonical bytes");
        assert_eq!(
            canonical,
            br#"{"integers":[9007199254740991,9007199254740992,-9007199254740992],"ratio":2.5}"#
        );
        assert_eq!(
            CheckedJson::from_canonical_bytes(&canonical, u64::MAX).expect("canonical readback"),
            payload
        );
    }

    #[trace("TC-048", "FR-038-AC-79", "FR-038-AC-514")]
    #[test]
    fn serializable_payload_refusal_preserves_literal_integer_construction() {
        use crate::checked_package::{CheckedNodeId, CheckedNodePathSegment, CheckedTerm};

        let type_id = CheckedNodeId {
            package: "example".into(),
            path: vec![CheckedNodePathSegment::Name("Integer".into())],
            ordinal: 0,
        };
        let term = CheckedTerm::literal_integer(type_id, 9_007_199_254_740_993);
        let CheckedTerm::Literal { value, .. } = term else {
            panic!("an integer constructor produces a literal");
        };
        assert_eq!(value.number_text(), Some("9007199254740993"));
        assert_eq!(value.as_u64(), Some(9_007_199_254_740_993));
        assert!(matches!(
            quire_canonical::to_vec(&value, Limits::new(u64::MAX)),
            Err(CanonicalError::IntegerMagnitudeAboveMaximum(
                9_007_199_254_740_993
            ))
        ));
    }

    #[trace("TC-472", "FR-038-AC-514")]
    #[test]
    fn handles_round_trip_compare_by_canonical_encoding_and_refuse_noncanonical_bytes() {
        let value = json!({"b": [1, 2.5, "x", null, true], "a": {"k": -7}});
        let left = CheckedJson::from_serializable(&value).expect("encodes");
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
        assert_ne!(
            left,
            CheckedJson::from_serializable(&json!([1])).expect("encodes")
        );
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

#[cfg(test)]
mod empty_aggregate_projection_tests {
    use super::super::model_members::EmptyAggregateBody;
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-472", "FR-038-AC-514")]
    #[test]
    fn closed_empty_aggregate_projection_borrows_the_shared_document_without_allocating() {
        for (value, expected) in [
            (json!({"term":"aggregate","members":[]}), true),
            (json!({"term":"aggregate","members":[null]}), false),
            (
                json!({"term":"aggregate","members":[],"extra":false}),
                false,
            ),
            (json!({"term":"aggregate"}), false),
            (json!({"members":[]}), false),
            (json!({"term":"binding","members":[]}), false),
            (json!({"term":1,"members":[]}), false),
            (json!({"term":"aggregate","members":null}), false),
            (json!(null), false),
            (json!([]), false),
        ] {
            let container =
                CheckedJson::from_value(&json!({"body":value})).expect("fixture document");
            let body_node = super::super::TermNode::of(&container)
                .get("body")
                .expect("body")
                .node_ref()
                .id();
            let body = CheckedJson::new(Arc::clone(container.document()), body_node);
            let mut observed = None;
            let measured = allocation_counter::measure(|| {
                let clone = body.clone();
                observed = Some(clone.is_empty_aggregate());
            });
            assert_eq!(observed, Some(expected), "{value}");
            assert!(Arc::ptr_eq(body.document(), container.document()));
            assert_eq!(measured.bytes_total, 0);
            assert_eq!(measured.count_total, 0);
        }
    }
}
