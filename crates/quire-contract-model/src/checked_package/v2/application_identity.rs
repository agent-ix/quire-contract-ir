// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-046 borrowed application drafts and candidate identities. The reader and
//! pre-read API share the preimage construction here and the encoder in `encode`.

use super::encode::{ApplicationNodePreimage, GroupPlace};
use super::{
    structural, BodyTerm, CheckedNodeId, CheckedNodeKind, CheckedNodeTag, CheckedTerm,
    ProtocolForm, RawSemanticNodeV2 as CheckedSemanticNodeV2,
};
use crate::checked_package::common::{body_term, NODE_DOMAIN};
use quire_walk::{walk, Children, Walk};
use serde_json::{json, Value};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::ops::ControlFlow;

/// A borrowed pre-read node. Its supplied key is a provisional selector, not
/// an authenticated semantic identity. Construction neither validates nor
/// mutates the node.
pub struct ApplicationNodeDraft<'a, Body = Value> {
    node: &'a super::CheckedSemanticNodeV2<Body>,
}

impl<Body> Copy for ApplicationNodeDraft<'_, Body> {}

impl<Body> Clone for ApplicationNodeDraft<'_, Body> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Body> ApplicationNodeDraft<'a, Body> {
    /// Borrows a node whose candidate application identity will be derived.
    pub fn new(node: &'a super::CheckedSemanticNodeV2<Body>) -> Self {
        Self { node }
    }
}

/// An immutable complete ordered draft snapshot. The caller supplies the
/// complete intended graph; this context verifies its keys and target without
/// asserting package admission. Raw drafts stay borrowed; typed drafts are
/// projected into owned raw nodes for the shared candidate encoder.
pub struct ApplicationIdentityContext<'a> {
    nodes: Cow<'a, [CheckedSemanticNodeV2]>,
    target: usize,
    group: Option<GroupPlace>,
}

impl<'a> ApplicationIdentityContext<'a> {
    /// Validates key uniqueness, the target's complete fields and its position
    /// among all nodes with its recursion label. Repeated keys refuse in graph
    /// order before target selection. Retained storage is constant in graph size
    /// for raw drafts and linear in graph size for typed drafts.
    pub fn try_new<Body: super::CandidateBody>(
        nodes: &'a [super::CheckedSemanticNodeV2<Body>],
        draft: ApplicationNodeDraft<'_, Body>,
    ) -> Result<Self, ApplicationIdentityError> {
        let nodes = Body::candidate_nodes(nodes);
        let draft_node = Body::candidate_node(draft.node);
        let index_error = || ApplicationIdentityError::ContextIndexOutOfRange {
            node_id: draft_node.node_id.clone(),
        };
        for (repeated, node) in nodes.iter().enumerate() {
            let repeated_position = u64::try_from(repeated).map_err(|_| index_error())?;
            for (first, previous) in nodes.iter().take(repeated).enumerate() {
                if previous.node_id == node.node_id {
                    return Err(ApplicationIdentityError::RepeatedKey {
                        node_id: node.node_id.clone(),
                        first_position: u64::try_from(first).map_err(|_| index_error())?,
                        repeated_position,
                    });
                }
            }
        }
        let target = nodes
            .iter()
            .position(|node| node.node_id == draft.node.node_id)
            .ok_or_else(|| ApplicationIdentityError::MissingTarget {
                node_id: draft.node.node_id.clone(),
            })?;
        let Some(node) = nodes.get(target) else {
            return Err(index_error());
        };
        if !equal_nodes(node, &draft_node) {
            return Err(ApplicationIdentityError::ContextMismatch {
                node_id: draft.node.node_id.clone(),
            });
        }
        let group = if let Some(label) = node.recursion_group.as_deref() {
            let mut size = 0_u64;
            let mut ordinal = 0_u64;
            for (position, member) in nodes.iter().enumerate() {
                if member.recursion_group.as_deref() == Some(label) {
                    if position == target {
                        ordinal = size;
                    }
                    size = size.checked_add(1).ok_or_else(index_error)?;
                }
            }
            Some(GroupPlace { size, ordinal })
        } else {
            None
        };
        Ok(Self {
            nodes,
            target,
            group,
        })
    }
}

/// A canonical application key for caller assembly, without an admission
/// verdict or a conversion into an admitted package.
///
/// ```compile_fail,E0451
/// use quire_contract_model::{CandidateApplicationNodeId, CheckedNodeId};
/// let _ = CandidateApplicationNodeId {
///     node_id: CheckedNodeId { domain: "d".into(), digest: "x".into() },
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateApplicationNodeId {
    node_id: CheckedNodeId,
}

impl CandidateApplicationNodeId {
    /// Returns the candidate key. Final package admission remains with the
    /// checked-package reader after the caller replaces provisional references.
    pub fn candidate_node_id(&self) -> &CheckedNodeId {
        &self.node_id
    }
}

/// A typed draft/context or canonical-encoding refusal, without a candidate.
#[derive(Debug, thiserror::Error)]
pub enum ApplicationIdentityError {
    /// The supplied body root is not an application.
    #[error("the draft is not an application")]
    NonApplication {
        /// Supplied provisional key.
        node_id: CheckedNodeId,
    },
    /// A grouped application requires its complete ordered context.
    #[error("the grouped draft has no context")]
    MissingContext {
        /// Supplied provisional key.
        node_id: CheckedNodeId,
    },
    /// The context snapshot contains no node with the supplied key.
    #[error("the snapshot has no draft target")]
    MissingTarget {
        /// Supplied provisional key.
        node_id: CheckedNodeId,
    },
    /// A repeated key, selected by earliest repeated position then predecessor.
    #[error("the snapshot repeats a node key at {repeated_position}")]
    RepeatedKey {
        /// Repeated key.
        node_id: CheckedNodeId,
        /// First matching graph position.
        first_position: u64,
        /// Earliest repeated graph position.
        repeated_position: u64,
    },
    /// The context target's complete fields differ from the supplied draft.
    #[error("the context does not match the draft")]
    ContextMismatch {
        /// Supplied provisional key.
        node_id: CheckedNodeId,
    },
    /// A graph position, group size or ordinal is not representable as u64.
    #[error("a context index is outside the supported representation")]
    ContextIndexOutOfRange {
        /// Supplied provisional key.
        node_id: CheckedNodeId,
    },
    /// The authoritative canonical encoder's refusal, including byte ceilings.
    #[error(transparent)]
    CanonicalEncoding(#[from] quire_canonical::Error),
}

/// Derives a pre-read candidate through the reader's authoritative preimage
/// and encoder. Context checks follow the application-root check, and precede
/// encoding. The byte limit bounds canonical output, not graph or copy storage.
pub fn derive_application_node_id<Body: super::CandidateBody>(
    draft: ApplicationNodeDraft<'_, Body>,
    context: Option<&ApplicationIdentityContext<'_>>,
    byte_limit: u64,
) -> Result<CandidateApplicationNodeId, ApplicationIdentityError> {
    let raw_node = Body::candidate_node(draft.node);
    let node = raw_node.as_ref();
    if body_term(&node.body) != Some(BodyTerm::Application) {
        return Err(ApplicationIdentityError::NonApplication {
            node_id: node.node_id.clone(),
        });
    }
    let group = match context {
        Some(context) => {
            let target = context.nodes.get(context.target);
            if !target.is_some_and(|target| equal_nodes(target, node)) {
                return Err(ApplicationIdentityError::ContextMismatch {
                    node_id: node.node_id.clone(),
                });
            }
            match (node.recursion_group.as_deref(), context.group) {
                (Some(label), Some(place)) => ApplicationGroup::Snapshot {
                    nodes: context.nodes.as_ref(),
                    label,
                    place,
                },
                (None, None) => ApplicationGroup::Keys(&[]),
                (Some(_), None) | (None, Some(_)) => {
                    return Err(ApplicationIdentityError::ContextMismatch {
                        node_id: node.node_id.clone(),
                    });
                }
            }
        }
        None => {
            if node.recursion_group.is_some() {
                return Err(ApplicationIdentityError::MissingContext {
                    node_id: node.node_id.clone(),
                });
            }
            ApplicationGroup::Keys(&[])
        }
    };
    let digest = application_digest(node, group, byte_limit)?;
    Ok(CandidateApplicationNodeId {
        node_id: CheckedNodeId {
            domain: NODE_DOMAIN.into(),
            digest: digest.into(),
        },
    })
}

/// Authenticated reader keys or a validated borrowed pre-read snapshot. This
/// adapter changes only selection storage, never the preimage field rules.
#[derive(Clone, Copy)]
pub(super) enum ApplicationGroup<'a> {
    Keys(&'a [&'a CheckedNodeId]),
    Indexed(&'a BTreeMap<&'a CheckedNodeId, usize>),
    Snapshot {
        nodes: &'a [CheckedSemanticNodeV2],
        label: &'a str,
        place: GroupPlace,
    },
}

impl ApplicationGroup<'_> {
    fn ordinal(self, target: &CheckedNodeId) -> Option<usize> {
        match self {
            Self::Keys(keys) => keys.iter().position(|key| **key == *target),
            Self::Indexed(ordinals) => ordinals.get(target).copied(),
            Self::Snapshot { nodes, label, .. } => nodes
                .iter()
                .filter(|node| node.recursion_group.as_deref() == Some(label))
                .position(|node| node.node_id == *target),
        }
    }

    fn is_empty(self) -> bool {
        match self {
            Self::Keys(keys) => keys.is_empty(),
            Self::Indexed(ordinals) => ordinals.is_empty(),
            Self::Snapshot { place, .. } => place.size == 0,
        }
    }
}

/// The sole application preimage construction, shared with reader admission
/// and the reader's authenticated group-local digest.
pub(super) fn application_preimage<'a>(
    node: &'a CheckedSemanticNodeV2,
    group: ApplicationGroup<'_>,
) -> Result<ApplicationNodePreimage<'a>, ApplicationIdentityError> {
    application_preimage_with_term(node, None, group)
}

fn application_preimage_with_term<'a>(
    node: &'a CheckedSemanticNodeV2,
    admitted: Option<&CheckedTerm>,
    group: ApplicationGroup<'_>,
) -> Result<ApplicationNodePreimage<'a>, ApplicationIdentityError> {
    let index_error = || ApplicationIdentityError::ContextIndexOutOfRange {
        node_id: node.node_id.clone(),
    };
    let recursion = match (node.recursion_group.as_ref(), group) {
        (None, _) => None,
        (Some(_), ApplicationGroup::Snapshot { place, .. }) => Some(place),
        (Some(_), ApplicationGroup::Keys(keys)) => {
            let ordinal = group.ordinal(&node.node_id).ok_or_else(|| {
                ApplicationIdentityError::ContextMismatch {
                    node_id: node.node_id.clone(),
                }
            })?;
            Some(GroupPlace {
                size: u64::try_from(keys.len()).map_err(|_| index_error())?,
                ordinal: u64::try_from(ordinal).map_err(|_| index_error())?,
            })
        }
        (Some(_), ApplicationGroup::Indexed(ordinals)) => {
            let ordinal = group.ordinal(&node.node_id).ok_or_else(|| {
                ApplicationIdentityError::ContextMismatch {
                    node_id: node.node_id.clone(),
                }
            })?;
            Some(GroupPlace {
                size: u64::try_from(ordinals.len()).map_err(|_| index_error())?,
                ordinal: u64::try_from(ordinal).map_err(|_| index_error())?,
            })
        }
    };
    Ok(ApplicationNodePreimage {
        node_tag: &node.node_tag,
        semantic_form: &node.semantic_form,
        semantic_type: &node.semantic_type,
        declaration: node.declaration.as_ref(),
        owner_model: if CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
            == Some(CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause))
        {
            node.owner_model.as_ref()
        } else {
            None
        },
        recursion,
        body: admitted.map_or_else(
            || group_references(&node.body, group),
            |term| group_references_admitted(&node.body, term, group),
        ),
    })
}

pub(super) fn application_digest(
    node: &CheckedSemanticNodeV2,
    group: ApplicationGroup<'_>,
    bytes: u64,
) -> Result<String, ApplicationIdentityError> {
    application_digest_with_term(node, None, group, bytes)
}

/// Reader path: the already-decoded admitted term selects every reference
/// and child, while the retained raw body supplies the exact wire preimage.
pub(super) fn application_digest_admitted(
    node: &CheckedSemanticNodeV2,
    term: &CheckedTerm,
    group: ApplicationGroup<'_>,
    bytes: u64,
) -> Result<String, ApplicationIdentityError> {
    application_digest_with_term(node, Some(term), group, bytes)
}

fn application_digest_with_term(
    node: &CheckedSemanticNodeV2,
    term: Option<&CheckedTerm>,
    group: ApplicationGroup<'_>,
    bytes: u64,
) -> Result<String, ApplicationIdentityError> {
    let preimage = match term {
        Some(term) => application_preimage_with_term(node, Some(term), group)?,
        None => application_preimage(node, group)?,
    };
    Ok(quire_canonical::sha256(&preimage, quire_canonical::Limits::new(bytes))?.to_string())
}

fn group_references_admitted<'a>(
    body: &'a Value,
    term: &CheckedTerm,
    group: ApplicationGroup<'_>,
) -> Cow<'a, Value> {
    if group.is_empty() {
        return Cow::Borrowed(body);
    }
    let mut rewritten = OwnedJson(copy_value(body));
    let ControlFlow::Continue(()) =
        walk(&mut TypedGroupRewrite { group }, (term, &mut rewritten.0));
    Cow::Owned(rewritten.take())
}

struct TypedGroupRewrite<'a> {
    group: ApplicationGroup<'a>,
}

impl<'t> Walk for TypedGroupRewrite<'t> {
    type Node = (&'t CheckedTerm, &'t mut Value);
    type Frame = ();
    type Stop = Infallible;

    fn enter(
        &mut self,
        (term, raw): Self::Node,
        children: &mut Children<'_, Self::Node>,
    ) -> ControlFlow<Infallible> {
        match term {
            CheckedTerm::Reference { target } => {
                if let Some(ordinal) = self.group.ordinal(target) {
                    let replacement = json!({ "term": "group_reference", "ordinal": ordinal });
                    quire_canonical::drop_value(std::mem::replace(raw, replacement));
                }
            }
            CheckedTerm::Application { arguments, .. } => {
                if let Some(Value::Array(values)) = raw.get_mut("arguments") {
                    children.extend(arguments.iter().zip(values.iter_mut()));
                }
            }
            CheckedTerm::Aggregate { members } => {
                if let Some(Value::Array(values)) = raw.get_mut("members") {
                    children.extend(members.iter().zip(values.iter_mut()));
                }
            }
            CheckedTerm::Binding { value, .. } => {
                if let Some(raw) = raw.get_mut("value") {
                    children.push((value.as_ref(), raw));
                }
            }
            CheckedTerm::OperationAnchorBody(body) => {
                if let Some(values) = raw.get_mut("members").and_then(Value::as_array_mut) {
                    children.extend(body.members.iter().zip(values.iter_mut()));
                }
            }
            CheckedTerm::Literal { .. }
            | CheckedTerm::DependencyReference { .. }
            | CheckedTerm::Frame { .. }
            | CheckedTerm::AbstractionRelation { .. }
            | CheckedTerm::Refinement { .. }
            | CheckedTerm::ExtensionForm { .. } => {}
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<Infallible> {
        ControlFlow::Continue(())
    }
}

fn group_references<'a>(term: &'a Value, group: ApplicationGroup<'_>) -> Cow<'a, Value> {
    if group.is_empty() {
        return Cow::Borrowed(term);
    }
    let mut rewritten = OwnedJson(copy_value(term));
    let ControlFlow::Continue(()) = walk(&mut GroupRewrite { group }, &mut rewritten.0);
    Cow::Owned(rewritten.take())
}

struct GroupRewrite<'a> {
    group: ApplicationGroup<'a>,
}

impl<'t> Walk for GroupRewrite<'t> {
    type Node = &'t mut Value;
    type Frame = ();
    type Stop = Infallible;

    fn enter(
        &mut self,
        term: &'t mut Value,
        children: &mut Children<'_, &'t mut Value>,
    ) -> ControlFlow<Infallible> {
        match body_term(term) {
            Some(BodyTerm::Reference) => {
                let ordinal = structural::reference_selector(term)
                    .and_then(|target| self.group.ordinal(&target));
                if let Some(ordinal) = ordinal {
                    let replacement = json!({ "term": "group_reference", "ordinal": ordinal });
                    quire_canonical::drop_value(std::mem::replace(term, replacement));
                }
            }
            Some(BodyTerm::Application) => {
                if let Some(Value::Array(arguments)) = term.get_mut("arguments") {
                    children.extend(arguments.iter_mut());
                }
            }
            Some(BodyTerm::Aggregate) => {
                if let Some(Value::Array(members)) = term.get_mut("members") {
                    children.extend(members.iter_mut());
                }
            }
            Some(BodyTerm::Binding) => children.extend(term.get_mut("value")),
            Some(
                BodyTerm::Literal
                | BodyTerm::DependencyReference
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation,
            )
            | None => {}
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<Infallible> {
        ControlFlow::Continue(())
    }
}

/// Every owned temporary, including a partially copied container, disposes of
/// its JSON through the canonical crate's nonrecursive ownership operation.
struct OwnedJson(Value);

impl OwnedJson {
    fn take(&mut self) -> Value {
        std::mem::take(&mut self.0)
    }
}

impl Drop for OwnedJson {
    fn drop(&mut self) {
        quire_canonical::drop_value(self.take());
    }
}

struct CopyArray(Vec<Value>);

impl Drop for CopyArray {
    fn drop(&mut self) {
        quire_canonical::drop_value(Value::Array(std::mem::take(&mut self.0)));
    }
}

struct CopyObject(serde_json::Map<String, Value>);

impl Drop for CopyObject {
    fn drop(&mut self) {
        quire_canonical::drop_value(Value::Object(std::mem::take(&mut self.0)));
    }
}

enum CopyFrame<'a> {
    Array(std::slice::Iter<'a, Value>, CopyArray),
    Object(serde_json::map::Iter<'a>, CopyObject, &'a str),
}

/// A depth-first copy of every JSON subtree, retaining one child iterator per
/// open container. No recursive clone or width-sized pending traversal stack.
fn copy_value(root: &Value) -> Value {
    let mut stack = Vec::new();
    let mut input = root;
    loop {
        let mut copied = match input {
            Value::Array(values) => {
                let mut remaining = values.iter();
                let container = CopyArray(Vec::new());
                if let Some(child) = remaining.next() {
                    stack.push(CopyFrame::Array(remaining, container));
                    input = child;
                    continue;
                }
                OwnedJson(Value::Array(Vec::new()))
            }
            Value::Object(values) => {
                let mut remaining = values.iter();
                let container = CopyObject(serde_json::Map::new());
                if let Some((key, child)) = remaining.next() {
                    stack.push(CopyFrame::Object(remaining, container, key));
                    input = child;
                    continue;
                }
                OwnedJson(Value::Object(serde_json::Map::new()))
            }
            Value::Null => OwnedJson(Value::Null),
            Value::Bool(value) => OwnedJson(Value::Bool(*value)),
            Value::Number(value) => OwnedJson(Value::Number(value.clone())),
            Value::String(value) => OwnedJson(Value::String(value.clone())),
        };
        loop {
            match stack.pop() {
                None => return copied.take(),
                Some(CopyFrame::Array(mut remaining, mut container)) => {
                    container.0.push(copied.take());
                    if let Some(child) = remaining.next() {
                        stack.push(CopyFrame::Array(remaining, container));
                        input = child;
                        break;
                    }
                    copied = OwnedJson(Value::Array(std::mem::take(&mut container.0)));
                }
                Some(CopyFrame::Object(mut remaining, mut container, key)) => {
                    container.0.insert(key.to_owned(), copied.take());
                    if let Some((key, child)) = remaining.next() {
                        stack.push(CopyFrame::Object(remaining, container, key));
                        input = child;
                        break;
                    }
                    copied = OwnedJson(Value::Object(std::mem::take(&mut container.0)));
                }
            }
        }
    }
}

fn equal_nodes(left: &CheckedSemanticNodeV2, right: &CheckedSemanticNodeV2) -> bool {
    left.node_id == right.node_id
        && left.schema_version == right.schema_version
        && left.node_tag == right.node_tag
        && left.semantic_form == right.semantic_form
        && left.semantic_type == right.semantic_type
        && left.dependencies == right.dependencies
        && left.occurrences == right.occurrences
        && left.recursion_group == right.recursion_group
        && left.nominal_identity_preimage == right.nominal_identity_preimage
        && left.declaration == right.declaration
        && left.owner == right.owner
        && left.owner_model == right.owner_model
        && equal_values(&left.body, &right.body)
}

enum EqualFrame<'a> {
    Array(std::iter::Zip<std::slice::Iter<'a, Value>, std::slice::Iter<'a, Value>>),
    Object(
        serde_json::map::Iter<'a>,
        &'a serde_json::Map<String, Value>,
    ),
}

impl<'a> EqualFrame<'a> {
    fn next(&mut self) -> Result<Option<(&'a Value, &'a Value)>, ()> {
        match self {
            Self::Array(values) => Ok(values.next()),
            Self::Object(values, right) => values
                .next()
                .map(|(key, left)| right.get(key).map(|right| (left, right)).ok_or(()))
                .transpose(),
        }
    }
}

fn equal_values<'a>(mut left: &'a Value, mut right: &'a Value) -> bool {
    let mut stack = Vec::new();
    loop {
        match (left, right) {
            (Value::Array(l), Value::Array(r)) => {
                if l.len() != r.len() {
                    return false;
                }
                stack.push(EqualFrame::Array(l.iter().zip(r.iter())));
            }
            (Value::Object(l), Value::Object(r)) => {
                if l.len() != r.len() {
                    return false;
                }
                stack.push(EqualFrame::Object(l.iter(), r));
            }
            (Value::Null, Value::Null) => {}
            (Value::Bool(l), Value::Bool(r)) if l == r => {}
            (Value::Number(l), Value::Number(r)) if l == r => {}
            (Value::String(l), Value::String(r)) if l == r => {}
            _ => return false,
        }
        loop {
            let Some(frame) = stack.last_mut() else {
                return true;
            };
            match frame.next() {
                Ok(Some((l, r))) => {
                    left = l;
                    right = r;
                    break;
                }
                Ok(None) => {
                    stack.pop();
                }
                Err(()) => return false,
            }
        }
    }
}
