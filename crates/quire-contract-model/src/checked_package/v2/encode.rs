//! Canonical encoding of the V2 wire types that hold a `serde_json::Value`.
//!
//! The V2 wire types encode to RFC 8785 canonical bytes through
//! `quire-canonical`, which owns the encoder. A type whose JSON nests to a depth
//! fixed by the type derives `FixedShape` where it is declared and encodes
//! through serde. The three types here cannot: a node `body` and a diagnostic's
//! `details` nest as deep as their input. Each implements
//! [`quire_canonical::Encode`] by pushing [`Writer`] events in its own field
//! order, delegating every fixed-depth member to [`Writer::serialize`] and
//! every `Value` to `quire-canonical`'s own `Encode` for `serde_json::Value`,
//! which walks from an explicit heap stack, so no native recursion follows the
//! input. [`Writer`] orders object members itself, so the bytes do not depend
//! on how a `Value`'s map is backed.
//!
//! The application node preimage, whose body is a `Value`, lives here too, and
//! so do the helpers the lowering's two preimages share. This module holds no
//! walker over a `Value` and no drop of one.
//!
//! Encoding adds no depth limit. The only refusals an encode returns are the
//! caller's byte ceiling and a number `quire-canonical` has no encoding for
//! (FR-038 "Canonical encoding of the wire types").

use super::{
    CheckedDeclaration, CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedNodeProjectionV2,
    CheckedPackageIdentityPreimageV2, CheckedSemanticGraphV2, CheckedSemanticNodeV2,
};
use crate::checked_package::shared::CheckedNodeId;
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde_json::Value;
use std::borrow::Cow;

/// `quire.application-node/v1`, the version of the application node preimage.
pub(super) const APPLICATION_NODE_VERSION: &str = "quire.application-node/v1";

/// A node's place in its recursion group: the group's size and the node's
/// ordinal among its members, in graph order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GroupPlace {
    pub(super) size: usize,
    pub(super) ordinal: usize,
}

/// QSpec FR-322's `application_node_preimage` of one node: `{version,
/// node_tag, semantic_form, semantic_type, declaration, recursion, body}`,
/// where `declaration` and `recursion` are `null` when the node has none. The
/// body is a `Value`, so the type implements [`Encode`] and not `FixedShape`.
pub(super) struct ApplicationNodePreimage<'a> {
    pub(super) node_tag: &'a str,
    pub(super) semantic_form: &'a str,
    pub(super) semantic_type: &'a CheckedNodeId,
    pub(super) declaration: Option<&'a CheckedDeclaration>,
    pub(super) recursion: Option<GroupPlace>,
    pub(super) body: Cow<'a, Value>,
}

impl Encode for ApplicationNodePreimage<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("version")?;
        writer.string(APPLICATION_NODE_VERSION)?;
        writer.name("node_tag")?;
        writer.string(self.node_tag)?;
        writer.name("semantic_form")?;
        writer.string(self.semantic_form)?;
        member(writer, "semantic_type", self.semantic_type)?;
        writer.name("declaration")?;
        match self.declaration {
            Some(declaration) => writer.serialize(declaration)?,
            None => writer.null()?,
        }
        writer.name("recursion")?;
        match self.recursion {
            Some(place) => {
                writer.begin_object()?;
                writer.name("size")?;
                writer.integer(integer(place.size)?)?;
                writer.name("ordinal")?;
                writer.integer(integer(place.ordinal)?)?;
                writer.end_object()?;
            }
            None => writer.null()?,
        }
        value(writer, "body", &self.body)?;
        writer.end_object()
    }
}

/// A count as the integer the writer takes.
fn integer(count: usize) -> Result<i128, Error> {
    i128::try_from(count).map_err(|_| Error::Internal {
        invariant: "a count fits i128",
    })
}

/// Writes the member `name` whose value has a depth fixed by its type.
pub(super) fn member<S, T>(writer: &mut Writer<'_, S>, name: &str, value: &T) -> Result<(), Error>
where
    S: Sink + ?Sized,
    T: FixedShape + ?Sized,
{
    writer.name(name)?;
    writer.serialize(value)
}

/// Writes the member `name` when `value` is present and nothing when it is
/// absent: the wire omits an absent optional member rather than writing `null`.
pub(super) fn present<S, T>(
    writer: &mut Writer<'_, S>,
    name: &str,
    value: &Option<T>,
) -> Result<(), Error>
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
pub(super) fn elements<S, T>(
    writer: &mut Writer<'_, S>,
    name: &str,
    items: &[T],
) -> Result<(), Error>
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

/// Writes the member `name` as `item`, a value that encodes itself.
pub(super) fn encoded<S, T>(writer: &mut Writer<'_, S>, name: &str, item: &T) -> Result<(), Error>
where
    S: Sink + ?Sized,
    T: Encode + ?Sized,
{
    writer.name(name)?;
    item.encode_into(writer)
}

/// Writes the member `name` as `body`, a JSON value of any depth, through
/// `quire-canonical`'s `Encode` for `serde_json::Value`.
pub(super) fn value<S>(writer: &mut Writer<'_, S>, name: &str, body: &Value) -> Result<(), Error>
where
    S: Sink + ?Sized,
{
    writer.name(name)?;
    body.encode_into(writer)
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
        value(writer, "body", &self.body)?;
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
        value(writer, "body", &self.body)?;
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
            detail.encode_into(writer)?;
        }
        writer.end_array()?;
        member(writer, "loci", &self.loci)?;
        writer.end_object()
    }
}
