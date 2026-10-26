//! Canonical encoding of the V2 wire types that hold a `serde_json::Value`.
//!
//! The V2 wire types encode to RFC 8785 canonical bytes through
//! `quire-canonical`, which owns the encoder. A type whose JSON nests to a depth
//! fixed by the type derives `FixedShape` where it is declared and encodes
//! through serde. The three retained wire types and the borrowed preimage here
//! cannot: a node `body` and a diagnostic's
//! `details` nest as deep as their input. Each implements
//! [`quire_canonical::Encode`] by pushing [`Writer`] events in its own field
//! order, delegating every fixed-depth member to [`Writer::serialize`] and
//! every `Value` to `quire-canonical`'s own `Encode` for `serde_json::Value`,
//! which walks from an explicit heap stack, so no native recursion follows the
//! input. [`Writer`] orders object members itself, so the bytes do not depend
//! on how a `Value`'s map is backed.
//!
//! The application node preimage, whose body is a `Value`, lives here too, and
//! so do the helpers the lowering's two preimages share. Grouped structural
//! bodies use borrowed event encoding; owned application rewrite bodies are
//! disposed of through `quire-canonical`'s nonrecursive ownership operation.
//!
//! Encoding adds no depth limit. The only refusals an encode returns are the
//! caller's byte ceiling and a number `quire-canonical` has no encoding for
//! (FR-038 "Canonical encoding of the wire types").

use super::{
    CheckedDeclaration, CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedNodeOwner,
    CheckedNodeProjectionV2, CheckedPackageIdentityPreimageRefV2, CheckedPackageIdentityPreimageV2,
    CheckedSemanticGraphV2, CheckedSemanticNodeV2, NominalIdentityPreimage,
};
use crate::checked_package::shared::CheckedNodeId;
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// `quire.application-node/v1`, the version of the application node preimage.
pub(super) const APPLICATION_NODE_VERSION: &str = "quire.application-node/v1";

/// A node's place in its recursion group: the group's size and the node's
/// ordinal among its members, in graph order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GroupPlace {
    pub(super) size: u64,
    pub(super) ordinal: u64,
}

/// QSpec FR-322's `application_node_preimage` of one node: `{version,
/// node_tag, semantic_form, semantic_type, declaration, recursion, body}`,
/// plus `owner_model` only for a protocol clause. `declaration` and
/// `recursion` are `null` when the node has none. The
/// body is a `Value`, so the type implements [`Encode`] and not `FixedShape`.
pub(super) struct ApplicationNodePreimage<'a> {
    pub(super) node_tag: &'a str,
    pub(super) semantic_form: &'a str,
    pub(super) semantic_type: &'a CheckedNodeId,
    pub(super) declaration: Option<&'a CheckedDeclaration>,
    pub(super) owner_model: Option<&'a CheckedNodeId>,
    pub(super) recursion: Option<GroupPlace>,
    pub(super) body: Cow<'a, Value>,
}

impl Drop for ApplicationNodePreimage<'_> {
    fn drop(&mut self) {
        // Group rewrites own arbitrary pre-read JSON, including on encoder refusal.
        if let Cow::Owned(body) = &mut self.body {
            quire_canonical::drop_value(std::mem::take(body));
        }
    }
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
        if let Some(owner_model) = self.owner_model {
            member(writer, "owner_model", owner_model)?;
        }
        writer.name("recursion")?;
        match self.recursion {
            Some(place) => {
                writer.begin_object()?;
                writer.name("size")?;
                writer.integer(i128::from(place.size))?;
                writer.name("ordinal")?;
                writer.integer(i128::from(place.ordinal))?;
                writer.end_object()?;
            }
            None => writer.null()?,
        }
        value(writer, "body", &self.body)?;
        writer.end_object()
    }
}

/// A borrowed body with references to group members encoded as wire ordinals.
/// The explicit event stack neither clones nor recursively drops input JSON.
pub(super) struct GroupBody<'a> {
    pub(super) body: &'a Value,
    pub(super) ordinals: &'a BTreeMap<&'a CheckedNodeId, usize>,
}

impl Encode for GroupBody<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        enum Event<'a> {
            Value(&'a Value),
            Name(&'a str),
            EndObject,
            EndArray,
        }
        let mut pending = vec![Event::Value(self.body)];
        while let Some(event) = pending.pop() {
            match event {
                Event::Name(name) => writer.name(name)?,
                Event::EndObject => writer.end_object()?,
                Event::EndArray => writer.end_array()?,
                Event::Value(value) => {
                    let ordinal = super::structural::reference_selector(value)
                        .and_then(|target| self.ordinals.get(&target).copied());
                    if let Some(ordinal) = ordinal {
                        writer.begin_object()?;
                        writer.name("term")?;
                        writer.string("group_reference")?;
                        writer.name("ordinal")?;
                        writer.integer(integer(ordinal)?)?;
                        writer.end_object()?;
                        continue;
                    }
                    match value {
                        Value::Object(members) => {
                            writer.begin_object()?;
                            pending.push(Event::EndObject);
                            for (name, member) in members.iter().rev() {
                                pending.push(Event::Value(member));
                                pending.push(Event::Name(name));
                            }
                        }
                        Value::Array(items) => {
                            writer.begin_array()?;
                            pending.push(Event::EndArray);
                            pending.extend(items.iter().rev().map(Event::Value));
                        }
                        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                            value.encode_into(writer)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// `quire.abstraction-relation-node/v1`, the version of the abstraction
/// relation node preimage (QSpec FR-451, FR-346).
pub(super) const ABSTRACTION_NODE_VERSION: &str = "quire.abstraction-relation-node/v1";

/// The dedicated closed refinement-node identity domain of QSpec FR-375.
pub(super) const REFINEMENT_NODE_VERSION: &str = "quire.refinement-node/v1";

/// The refinement preimage includes the concrete model semantic type even
/// when all six row arrays are empty.
pub(super) struct RefinementNodePreimage<'a> {
    pub(super) semantic_type: &'a CheckedNodeId,
    pub(super) body: &'a Value,
}

impl Encode for RefinementNodePreimage<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("version")?;
        writer.string(REFINEMENT_NODE_VERSION)?;
        member(writer, "semantic_type", self.semantic_type)?;
        value(writer, "body", self.body)?;
        writer.end_object()
    }
}

/// QSpec FR-451's preimage of a `correspondence`/`abstraction_relation`
/// node: `{version, body}`. The node key is the SHA-256 of its canonical
/// bytes, so every binding of the body is part of the key.
pub(super) struct AbstractionNodePreimage<'a> {
    pub(super) body: &'a Value,
}

impl Encode for AbstractionNodePreimage<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("version")?;
        writer.string(ABSTRACTION_NODE_VERSION)?;
        value(writer, "body", self.body)?;
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
        writer.end_object()
    }
}

/// The seven wire selections; graph nodes are only encoded under semantic_graph.
#[derive(serde::Serialize, FixedShape)]
pub(super) struct SelectionFields<'a> {
    version: &'a str,
    edition: &'a crate::checked_package::shared::CheckedSelection,
    profile_selections: &'a [crate::checked_package::shared::CheckedSelection],
    definition_selections: &'a [crate::checked_package::shared::CheckedArtifactRef],
    model_selections: &'a [super::CheckedDomainPackageRef],
    required_features: &'a [Box<str>],
    dependency_selections: &'a [super::CheckedDependencySelection],
}

impl<'a, Body> From<&CheckedPackageIdentityPreimageRefV2<'a, Body>> for SelectionFields<'a> {
    fn from(value: &CheckedPackageIdentityPreimageRefV2<'a, Body>) -> Self {
        Self {
            version: value.version,
            edition: value.edition,
            profile_selections: value.profile_selections,
            definition_selections: value.definition_selections,
            model_selections: value.model_selections,
            required_features: value.required_features,
            dependency_selections: value.dependency_selections,
        }
    }
}

impl<Body: Encode> Encode for CheckedPackageIdentityPreimageRefV2<'_, Body> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("version")?;
        writer.string(self.version)?;
        member(writer, "edition", self.edition)?;
        member(writer, "profile_selections", self.profile_selections)?;
        member(writer, "definition_selections", self.definition_selections)?;
        member(writer, "model_selections", self.model_selections)?;
        member(writer, "required_features", self.required_features)?;
        member(writer, "dependency_selections", self.dependency_selections)?;
        writer.name("identity_projection")?;
        writer.begin_array()?;
        for node in self.identity_projection {
            ProjectionFields::from_node(node).encode_into(writer)?;
        }
        writer.end_array()?;
        writer.end_object()
    }
}

/// The fields common to the retained reader projection and the producer's
/// borrowed graph-node projection. Source occurrences are intentionally absent.
struct ProjectionFields<'a, Body> {
    node_id: &'a CheckedNodeId,
    schema_version: &'a str,
    node_tag: &'a str,
    semantic_form: &'a str,
    semantic_type: &'a CheckedNodeId,
    dependencies: &'a [CheckedNodeId],
    recursion_group: Option<&'a str>,
    nominal_identity_preimage: Option<&'a NominalIdentityPreimage>,
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
            nominal_identity_preimage: node.nominal_identity_preimage.as_ref(),
            declaration: node.declaration.as_ref(),
            owner: node.owner.as_ref(),
            owner_model: node.owner_model.as_ref(),
            body: &node.body,
        }
    }

    fn from_node(node: &'a CheckedSemanticNodeV2<Body>) -> Self {
        Self {
            node_id: &node.node_id,
            schema_version: &node.schema_version,
            node_tag: &node.node_tag,
            semantic_form: &node.semantic_form,
            semantic_type: &node.semantic_type,
            dependencies: &node.dependencies,
            recursion_group: node.recursion_group.as_deref(),
            nominal_identity_preimage: node.nominal_identity_preimage.as_ref(),
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
        member(writer, "node_id", self.node_id)?;
        member(writer, "schema_version", self.schema_version)?;
        member(writer, "node_tag", self.node_tag)?;
        member(writer, "semantic_form", self.semantic_form)?;
        member(writer, "semantic_type", self.semantic_type)?;
        member(writer, "dependencies", self.dependencies)?;
        if let Some(group) = self.recursion_group {
            member(writer, "recursion_group", group)?;
        }
        if let Some(preimage) = self.nominal_identity_preimage {
            member(writer, "nominal_identity_preimage", preimage)?;
        }
        if let Some(declaration) = self.declaration {
            member(writer, "declaration", declaration)?;
        }
        if let Some(owner) = self.owner {
            member(writer, "owner", owner)?;
        }
        if let Some(owner_model) = self.owner_model {
            member(writer, "owner_model", owner_model)?;
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
        present(writer, "owner", &self.owner)?;
        present(writer, "owner_model", &self.owner_model)?;
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
