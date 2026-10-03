//! Canonical encoding of the V2 wire types that hold a `serde_json::Value`.
//!
//! The V2 wire types encode to RFC 8785 canonical bytes through
//! `quire-canonical`, which owns the encoder. A type whose JSON nests to a depth
//! fixed by the type derives `FixedShape` where it is declared and encodes
//! through serde. The three types here cannot: a node `body` and a diagnostic's
//! `details` nest as deep as their input. Each implements
//! [`quire_canonical::Encode`] by pushing [`Writer`] events in its own field
//! order, delegating every fixed-depth member to [`Writer::serialize`] and
//! writing each `Value` from an explicit heap stack ([`write_value`]), so no
//! native recursion follows the input. [`Writer`] orders object members itself,
//! so the bytes do not depend on how a `Value`'s map is backed.
//!
//! Encoding adds no depth limit. The only refusals an encode returns are the
//! caller's byte ceiling and a number `quire-canonical` has no encoding for
//! (FR-038 "Canonical encoding of the wire types").

use super::{
    CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedNodeProjectionV2,
    CheckedPackageIdentityPreimageV2, CheckedSemanticGraphV2, CheckedSemanticNodeV2,
};
use quire_canonical::{Encode, Error, FixedShape, Limits, Sink, Writer};
use serde_json::{map, Number, Value};
use std::slice;

/// Writes the member `name` whose value has a depth fixed by its type.
fn member<S, T>(writer: &mut Writer<'_, S>, name: &str, value: &T) -> Result<(), Error>
where
    S: Sink + ?Sized,
    T: FixedShape + ?Sized,
{
    writer.name(name)?;
    writer.serialize(value)
}

/// Writes the member `name` when `value` is present and nothing when it is
/// absent: the wire omits an absent optional member rather than writing `null`.
fn present<S, T>(writer: &mut Writer<'_, S>, name: &str, value: &Option<T>) -> Result<(), Error>
where
    S: Sink + ?Sized,
    T: FixedShape,
{
    match value {
        Some(value) => member(writer, name, value),
        None => Ok(()),
    }
}

/// Writes the member `name` as an array of values that encode themselves.
fn elements<S, T>(writer: &mut Writer<'_, S>, name: &str, items: &[T]) -> Result<(), Error>
where
    S: Sink + ?Sized,
    T: Encode,
{
    writer.name(name)?;
    writer.begin_array()?;
    for item in items {
        item.encode_into(writer)?;
    }
    writer.end_array()
}

/// Writes the member `name` as `value`, a JSON value of any depth.
fn body<S>(writer: &mut Writer<'_, S>, name: &str, value: &Value) -> Result<(), Error>
where
    S: Sink + ?Sized,
{
    writer.name(name)?;
    write_value(writer, value)
}

/// One container of a [`Value`] that is still open, with the members it has
/// yet to write.
enum Open<'v> {
    Array(slice::Iter<'v, Value>),
    Object(map::Iter<'v>),
}

/// Writes `root` as one JSON value, from an explicit heap stack of the
/// containers still open and never by recursion, so a value of any depth
/// encodes on any thread stack. An array is `begin_array`, its elements and
/// `end_array`; an object is `begin_object`, a `name` and value per member and
/// `end_object`; a scalar is the matching scalar event.
fn write_value<S>(writer: &mut Writer<'_, S>, root: &Value) -> Result<(), Error>
where
    S: Sink + ?Sized,
{
    let mut open: Vec<Open<'_>> = Vec::new();
    let mut next = Some(root);
    loop {
        if let Some(value) = next.take() {
            match value {
                Value::Null => writer.null()?,
                Value::Bool(flag) => writer.bool(*flag)?,
                Value::Number(number) => write_number(writer, number)?,
                Value::String(text) => writer.string(text)?,
                Value::Array(items) => {
                    writer.begin_array()?;
                    open.push(Open::Array(items.iter()));
                }
                Value::Object(members) => {
                    writer.begin_object()?;
                    open.push(Open::Object(members.iter()));
                }
            }
        }
        match open.last_mut() {
            None => return Ok(()),
            Some(Open::Array(items)) => match items.next() {
                Some(item) => next = Some(item),
                None => {
                    writer.end_array()?;
                    open.pop();
                }
            },
            Some(Open::Object(members)) => match members.next() {
                Some((name, member)) => {
                    writer.name(name)?;
                    next = Some(member);
                }
                None => {
                    writer.end_object()?;
                    open.pop();
                }
            },
        }
    }
}

/// The canonical bytes of `value`, a JSON value of any depth, under a byte
/// ceiling of `ceiling`. The reader compares them with the bytes it was given.
pub(in crate::checked_package) fn value_to_vec(
    value: &Value,
    ceiling: u64,
) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    let mut writer = Writer::new(&mut bytes, Limits::new(ceiling));
    write_value(&mut writer, value)?;
    writer.finish()?;
    Ok(bytes)
}

/// Writes a JSON number by its kind: an `i64` or `u64` through
/// [`Writer::integer`], which refuses a magnitude past 2^53, and any other
/// number through [`Writer::number`] as the `f64` it denotes.
fn write_number<S>(writer: &mut Writer<'_, S>, number: &Number) -> Result<(), Error>
where
    S: Sink + ?Sized,
{
    if let Some(integer) = number.as_i64() {
        writer.integer(i128::from(integer))
    } else if let Some(integer) = number.as_u64() {
        writer.integer(i128::from(integer))
    } else if let Some(float) = number.as_f64() {
        writer.number(float)
    } else {
        Err(Error::Serialize(String::from(
            "a JSON number has no integer or f64 value",
        )))
    }
}

impl Encode for CheckedPackageIdentityPreimageV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        member(writer, "version", &self.version)?;
        member(writer, "edition", &self.edition)?;
        member(writer, "profile_selections", &self.profile_selections)?;
        member(writer, "definition_selections", &self.definition_selections)?;
        member(writer, "model_selections", &self.model_selections)?;
        member(writer, "required_features", &self.required_features)?;
        member(writer, "dependency_selections", &self.dependency_selections)?;
        elements(writer, "identity_projection", &self.identity_projection)?;
        writer.end_object()
    }
}

impl Encode for CheckedNodeProjectionV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        member(writer, "node_id", &self.node_id)?;
        member(writer, "schema_version", &self.schema_version)?;
        member(writer, "node_tag", &self.node_tag)?;
        member(writer, "semantic_form", &self.semantic_form)?;
        member(writer, "semantic_type", &self.semantic_type)?;
        member(writer, "dependencies", &self.dependencies)?;
        present(writer, "recursion_group", &self.recursion_group)?;
        present(
            writer,
            "nominal_identity_preimage",
            &self.nominal_identity_preimage,
        )?;
        present(writer, "declaration", &self.declaration)?;
        body(writer, "body", &self.body)?;
        writer.end_object()
    }
}

impl Encode for CheckedSemanticGraphV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        member(writer, "graph_version", &self.graph_version)?;
        elements(writer, "nodes", &self.nodes)?;
        writer.end_object()
    }
}

impl Encode for CheckedSemanticNodeV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        member(writer, "node_id", &self.node_id)?;
        member(writer, "schema_version", &self.schema_version)?;
        member(writer, "node_tag", &self.node_tag)?;
        member(writer, "semantic_form", &self.semantic_form)?;
        member(writer, "semantic_type", &self.semantic_type)?;
        member(writer, "dependencies", &self.dependencies)?;
        member(writer, "occurrences", &self.occurrences)?;
        present(writer, "recursion_group", &self.recursion_group)?;
        present(
            writer,
            "nominal_identity_preimage",
            &self.nominal_identity_preimage,
        )?;
        present(writer, "declaration", &self.declaration)?;
        body(writer, "body", &self.body)?;
        writer.end_object()
    }
}

impl Encode for CheckedDiagnosticsV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        member(writer, "catalog", &self.catalog)?;
        elements(writer, "entries", &self.entries)?;
        writer.end_object()
    }
}

impl Encode for CheckedDiagnosticV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        member(writer, "stage", &self.stage)?;
        member(writer, "code", &self.code)?;
        member(writer, "cause_tag", &self.cause_tag)?;
        writer.name("details")?;
        writer.begin_array()?;
        for detail in &self.details {
            write_value(writer, detail)?;
        }
        writer.end_array()?;
        member(writer, "loci", &self.loci)?;
        writer.end_object()
    }
}
