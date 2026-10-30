//! Canonical encoding of the V2 wire types that hold a `serde_json::Value`.
//!
//! The V2 wire types encode to RFC 8785 canonical bytes through
//! `quire-canonical`, which owns the encoder. A type whose JSON nests to a depth
//! fixed by the type derives `FixedShape` where it is declared and encodes
//! through serde. The retained wire types here
//! cannot: a node `body` and a diagnostic's
//! `details` nest as deep as their input. Each implements
//! [`quire_canonical::Encode`] by pushing [`Writer`] events in its own field
//! order, delegating every fixed-depth member to [`Writer::serialize`] and
//! every `Value` to `quire-canonical`'s own `Encode` for `serde_json::Value`,
//! which walks from an explicit heap stack, so no native recursion follows the
//! input. [`Writer`] orders object members itself, so the bytes do not depend
//! on how a `Value`'s map is backed.
//!
//! Encoding adds no depth limit. The only refusals an encode returns are the
//! caller's byte ceiling and a number `quire-canonical` has no encoding for
//! (FR-038 "Canonical encoding of the wire types").

use super::{
    CheckedDeclaration, CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedNodeOwner,
    CheckedNodeProjectionV2, CheckedSemanticGraphV2, CheckedSemanticNodeV2,
};
use crate::checked_package::shared::CheckedNodeId;
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};

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

/// Writes a present optional member whose value has input-dependent depth.
pub(super) fn present_encoded<S, T>(
    writer: &mut Writer<'_, S>,
    name: &str,
    value: &Option<T>,
) -> Result<(), Error>
where
    S: Sink + ?Sized,
    T: Encode,
{
    if let Some(value) = value {
        encoded(writer, name, value)?;
    }
    Ok(())
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

/// The fields of one retained graph-node view used by lowering.
struct ProjectionFields<'a, Body> {
    node_id: &'a CheckedNodeId,
    schema_version: &'a str,
    node_tag: &'a str,
    semantic_form: &'a str,
    semantic_type: &'a CheckedNodeId,
    dependencies: &'a [CheckedNodeId],
    recursion_group: Option<&'a str>,
    declaration: Option<&'a CheckedDeclaration>,
    owner: Option<&'a CheckedNodeOwner>,
    owner_model: Option<&'a CheckedNodeId>,
    body: &'a Body,
}

impl<'a, Body> ProjectionFields<'a, Body> {
    fn from_owned(node: &'a CheckedNodeProjectionV2<Body>) -> Self {
        Self {
            node_id: &node.node_id,
            schema_version: &node.schema_version,
            node_tag: &node.node_tag,
            semantic_form: &node.semantic_form,
            semantic_type: &node.semantic_type,
            dependencies: &node.dependencies,
            recursion_group: node.recursion_group.as_deref(),
            declaration: node.declaration.as_ref(),
            owner: node.owner.as_ref(),
            owner_model: node.owner_model.as_ref(),
            body: &node.body,
        }
    }
}

impl<Body: Encode> Encode for ProjectionFields<'_, Body> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        encoded(writer, "node_id", self.node_id)?;
        member(writer, "schema_version", self.schema_version)?;
        member(writer, "node_tag", self.node_tag)?;
        member(writer, "semantic_form", self.semantic_form)?;
        encoded(writer, "semantic_type", self.semantic_type)?;
        elements(writer, "dependencies", self.dependencies)?;
        if let Some(group) = self.recursion_group {
            member(writer, "recursion_group", group)?;
        }
        if let Some(declaration) = self.declaration {
            member(writer, "declaration", declaration)?;
        }
        if let Some(owner) = self.owner {
            member(writer, "owner", owner)?;
        }
        if let Some(owner_model) = self.owner_model {
            encoded(writer, "owner_model", owner_model)?;
        }
        encoded(writer, "body", self.body)?;
        writer.end_object()
    }
}

impl<Body: Encode> Encode for CheckedNodeProjectionV2<Body> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        ProjectionFields::from_owned(self).encode_into(writer)
    }
}

impl<Body: Encode> Encode for CheckedSemanticGraphV2<Body> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        member(writer, "graph_version", &self.graph_version)?;
        elements(writer, "nodes", &self.nodes)?;
        writer.end_object()
    }
}

impl<Body: Encode> Encode for CheckedSemanticNodeV2<Body> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        encoded(writer, "node_id", &self.node_id)?;
        member(writer, "schema_version", &self.schema_version)?;
        member(writer, "node_tag", &self.node_tag)?;
        member(writer, "semantic_form", &self.semantic_form)?;
        encoded(writer, "semantic_type", &self.semantic_type)?;
        elements(writer, "dependencies", &self.dependencies)?;
        member(writer, "occurrences", &self.occurrences)?;
        present(writer, "recursion_group", &self.recursion_group)?;
        present(writer, "declaration", &self.declaration)?;
        present(writer, "owner", &self.owner)?;
        present_encoded(writer, "owner_model", &self.owner_model)?;
        encoded(writer, "body", &self.body)?;
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
