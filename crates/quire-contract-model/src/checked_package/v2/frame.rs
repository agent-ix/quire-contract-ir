//! QSpec FR-340 frame bodies: the closed `{term: "frame", modifies, creates,
//! deletes, deletes_scoped}` body of a `state`/`frame` node, and the frame step of the
//! reader order.
//!
//! [`read_frame_body`] is the shape check the per-node body loop runs: each
//! member an array of distinct entries, `creates` and `deletes` of node keys
//! and `modifies` of `FrameModifiesEntry` values, `{kind: "relationship",
//! declaration}` or `{kind: "field", declaration, name, scope}`, and
//! `deletes_scoped` of receiver or named parameter entries. A shape defect
//! refuses as `invalid_semantic_graph` at the entry, a repeated entry at its second
//! occurrence.
//!
//! [`validate_frame_semantics`] is the frame step, run after every
//! graph-shape and declaration refusal. For each frame node, in
//! ascending structural node-id order, it collects every meaning-join defect:
//!
//! - the frame's `semantic_type` names no node (`missing-name`) or a node
//!   that is not an operation-bearing object type, process or state machine
//!   (`malformed-declaration`);
//! - an entry whose node key is not among the frame's `dependencies` or
//!   names no node (`missing-name`), or names a node its kind or member
//!   cannot carry (`malformed-declaration`);
//! - a field entry whose name does not resolve among the fields of its
//!   declaring node (FR-322 "Model-owned members" steps 2 and 3): the owner
//!   refusal, `missing-name` for no field of that name, `ambiguous-name` for
//!   two.
//! - a receiver-scoped field or deletion on a process or state-machine frame
//!   (`malformed-declaration`), after member joins and before order checks.
//!
//! A field entry's name resolves through the selected domain package only
//! when its declaring node is a model declaration node (the scope FR-038's
//! "Model-owned members" gives the resolution). A declaring node that carries
//! its own `declaration` is declared by this package's source, names no
//! member in its body, and is not resolved against any domain package.
//!
//! The reported meaning defect is the least by (position, entry declaration id,
//! field name), positions ordered `semantic_type`, `modifies`, `creates`,
//! `deletes`. A conflicting pair of field scopes follows other meaning defects
//! and precedes order checks. With no meaning defect, a member not strictly ascending by
//! its order key refuses as `invalid_semantic_graph` at the frame body,
//! located at the frame node. The first frame carrying a defect is reported.

use super::graph_storage::GraphIndex;
use super::identity::is_identifier;
use super::model_members::{MemberKind, ModelOwners, ModelRefusal};
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    BodyTerm, CheckedFieldScope, CheckedFrameModification, CheckedNodeKind, CheckedScopedDeletion,
    CheckedSemanticNodeV2, CheckedTerm, FrameEntryKind, ModelForm, RelationForm, WorkMeter,
};
use crate::checked_package::common::{
    body_term, exact_members, visit_reference, ReferenceMember, Trail, ValidationFailure,
};
use crate::checked_package::shared::CheckedPackageRefusalCause;
use crate::checked_package::shared::{CheckedNodeId, CheckedPackageRefusalCode};
use serde_json::Value;
use std::collections::BTreeSet;

/// One `modifies` entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ModifiesEntry {
    /// `{kind: "relationship", declaration}`.
    Relationship { declaration: CheckedNodeId },
    /// `{kind: "field", declaration, name, scope}`.
    Field {
        declaration: CheckedNodeId,
        name: Box<str>,
        scope: FieldScope,
    },
}

/// Scope of a field write grant.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FieldScope {
    EveryObject,
    Receiver,
}

impl FieldScope {
    #[ix_trace_rs::string_edge]
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "every-object" => Some(Self::EveryObject),
            "receiver" => Some(Self::Receiver),
            _ => None,
        }
    }
}

/// One scoped deletion grant.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum DeletesScopedEntry {
    Receiver,
    Parameter { name: Box<str> },
}

impl ModifiesEntry {
    fn declaration(&self) -> &CheckedNodeId {
        match self {
            Self::Relationship { declaration } | Self::Field { declaration, .. } => declaration,
        }
    }

    /// The field name, empty for a relationship entry.
    fn name(&self) -> &str {
        match self {
            Self::Relationship { .. } => "",
            Self::Field { name, .. } => name,
        }
    }

    /// FR-340's order key: the declaration id, field name, then scope.
    fn order_key(&self) -> (&CheckedNodeId, &str, Option<FieldScope>) {
        let scope = match self {
            Self::Relationship { .. } => None,
            Self::Field { scope, .. } => Some(*scope),
        };
        (self.declaration(), self.name(), scope)
    }
}

/// A frame body as the wire carries it, entries in wire order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct FrameBody {
    pub(super) modifies: Vec<ModifiesEntry>,
    pub(super) creates: Vec<CheckedNodeId>,
    pub(super) deletes: Vec<CheckedNodeId>,
    pub(super) deletes_scoped: Vec<DeletesScopedEntry>,
}

impl FrameBody {
    /// Retains the already validated frame as the public typed term. This
    /// avoids a second generic JSON decode of nested structural node IDs.
    pub(super) fn into_checked_term_legacy(self) -> CheckedTerm {
        let modifies = self
            .modifies
            .into_iter()
            .map(|entry| match entry {
                ModifiesEntry::Relationship { declaration } => {
                    CheckedFrameModification::Relationship { declaration }
                }
                ModifiesEntry::Field {
                    declaration,
                    name,
                    scope,
                } => CheckedFrameModification::Field {
                    declaration,
                    name,
                    scope: match scope {
                        FieldScope::EveryObject => CheckedFieldScope::EveryObject,
                        FieldScope::Receiver => CheckedFieldScope::Receiver,
                    },
                },
            })
            .collect();
        let deletes_scoped = self
            .deletes_scoped
            .into_iter()
            .map(|entry| match entry {
                DeletesScopedEntry::Receiver => CheckedScopedDeletion::Receiver,
                DeletesScopedEntry::Parameter { name } => CheckedScopedDeletion::Parameter { name },
            })
            .collect();
        CheckedTerm::Frame {
            modifies,
            creates: self.creates,
            deletes: self.deletes,
            deletes_scoped,
        }
    }

    /// Copies the retained typed representation using the same caller meter.
    pub(super) fn checked_term(
        &self,
        meter: &mut WorkMeter,
    ) -> Result<CheckedTerm, ValidationFailure> {
        let mut modifies = meter.vec(self.modifies.len())?;
        for entry in &self.modifies {
            modifies.push(match entry {
                ModifiesEntry::Relationship { declaration } => {
                    CheckedFrameModification::Relationship {
                        declaration: meter.clone_id(declaration)?,
                    }
                }
                ModifiesEntry::Field {
                    declaration,
                    name,
                    scope,
                } => CheckedFrameModification::Field {
                    declaration: meter.clone_id(declaration)?,
                    name: meter.boxed_str(name)?,
                    scope: match scope {
                        FieldScope::EveryObject => CheckedFieldScope::EveryObject,
                        FieldScope::Receiver => CheckedFieldScope::Receiver,
                    },
                },
            });
        }
        let mut creates = meter.vec(self.creates.len())?;
        for id in &self.creates {
            creates.push(meter.clone_id(id)?);
        }
        let mut deletes = meter.vec(self.deletes.len())?;
        for id in &self.deletes {
            deletes.push(meter.clone_id(id)?);
        }
        let mut deletes_scoped = meter.vec(self.deletes_scoped.len())?;
        for entry in &self.deletes_scoped {
            deletes_scoped.push(match entry {
                DeletesScopedEntry::Receiver => CheckedScopedDeletion::Receiver,
                DeletesScopedEntry::Parameter { name } => CheckedScopedDeletion::Parameter {
                    name: meter.boxed_str(name)?,
                },
            });
        }
        Ok(CheckedTerm::Frame {
            modifies,
            creates,
            deletes,
            deletes_scoped,
        })
    }
}

/// A frame borrowed from the already admitted typed term table.
pub(super) struct FrameView<'a> {
    pub(super) modifies: &'a [CheckedFrameModification],
    pub(super) deletes_scoped: &'a [CheckedScopedDeletion],
    work: u64,
}
impl<'a> FrameView<'a> {
    pub(super) fn of(term: &'a CheckedTerm) -> Option<Self> {
        let CheckedTerm::Frame {
            modifies,
            creates,
            deletes,
            deletes_scoped,
        } = term
        else {
            return None;
        };
        Some(Self {
            modifies,
            deletes_scoped,
            work: 1_u64
                .saturating_add(modifies.len() as u64)
                .saturating_add(creates.len() as u64)
                .saturating_add(deletes.len() as u64)
                .saturating_add(deletes_scoped.len() as u64),
        })
    }
    pub(super) fn work(&self) -> u64 {
        self.work
    }
}

/// Reads a `state`/`frame` node's body at `at`, returning it and its work:
/// one unit for the body and one per entry.
pub(super) fn read_frame_body_legacy(
    body: &Value,
    at: &Trail<'_>,
) -> Result<(FrameBody, u64), ValidationFailure> {
    let invalid = |position: &Trail<'_>| {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            position.pointer(),
        )
    };
    let Value::Object(object) = body else {
        return Err(invalid(at));
    };
    if !exact_members(
        object,
        &["term", "modifies", "creates", "deletes", "deletes_scoped"],
    ) {
        return Err(invalid(at));
    }
    if body_term(body) != Some(BodyTerm::Frame) {
        return Err(invalid(&at.key("term")));
    }
    let entries = |key: &str| -> Result<&[Value], ValidationFailure> {
        match object.get(key) {
            Some(Value::Array(values)) => Ok(values.as_slice()),
            _ => Err(invalid(&at.key(key))),
        }
    };
    let modifies_at = at.key("modifies");
    let modifies = entries("modifies")?
        .iter()
        .enumerate()
        .map(|(index, entry)| read_modifies_entry(entry, &modifies_at.index(index)))
        .collect::<Result<Vec<_>, _>>()?;
    reject_repeat(&modifies, &modifies_at)?;
    let creates_at = at.key("creates");
    let creates = read_node_keys(entries("creates")?, &creates_at)?;
    reject_repeat(&creates, &creates_at)?;
    let deletes_at = at.key("deletes");
    let deletes = read_node_keys(entries("deletes")?, &deletes_at)?;
    reject_repeat(&deletes, &deletes_at)?;
    let deletes_scoped_at = at.key("deletes_scoped");
    let deletes_scoped = entries("deletes_scoped")?
        .iter()
        .enumerate()
        .map(|(index, entry)| read_deletes_scoped_entry(entry, &deletes_scoped_at.index(index)))
        .collect::<Result<Vec<_>, _>>()?;
    reject_repeat(&deletes_scoped, &deletes_scoped_at)?;
    let count = |len: usize| u64::try_from(len).unwrap_or(u64::MAX);
    let work = 1_u64
        .saturating_add(count(modifies.len()))
        .saturating_add(count(creates.len()))
        .saturating_add(count(deletes.len()));
    let work = work.saturating_add(count(deletes_scoped.len()));
    Ok((
        FrameBody {
            modifies,
            creates,
            deletes,
            deletes_scoped,
        },
        work,
    ))
}

/// Reads the original checked document directly, charging each owned block.
pub(super) fn read_frame_body(
    body: super::term_view::TermNode<'_>,
    position: usize,
    row: &[u64],
    meter: &mut WorkMeter,
) -> Result<(FrameBody, u64), ValidationFailure> {
    use super::native_join_budget::PackageReadStage;
    use super::stage_storage::{self, Path, Refusal, Site};
    use super::term_view::{NodeIdRead, TermNode};
    let root = Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
        .key("body");
    let site = Site::new(PackageReadStage::Graph, position, "state", root);
    let invalid = |meter: &mut WorkMeter, path, code| {
        stage_storage::refuse(meter, &site, Refusal::new(code, path))
    };
    let code = CheckedPackageRefusalCode::InvalidSemanticGraph;
    if !body.exact_members(&["term", "modifies", "creates", "deletes", "deletes_scoped"]) {
        return Err(invalid(meter, root, code));
    }
    if body.kind() != Some(BodyTerm::Frame) {
        return Err(invalid(meter, root.key("term"), code));
    }
    let array = |key: &'static str, meter: &mut WorkMeter| {
        body.get(key)
            .and_then(TermNode::items)
            .ok_or_else(|| invalid(meter, root.key(key), code))
    };
    let read_id = |value: Option<TermNode<'_>>, meter: &mut WorkMeter, at| {
        let Some(value) = value else {
            return Err(invalid(meter, at, code));
        };
        let id_site = super::read_charge::Site::new(PackageReadStage::Graph, position, "state");
        match value
            .node_id_in(meter, id_site, row)
            .map_err(|_| super::read_charge::ReadStorage::denial(meter))?
        {
            NodeIdRead::Id(id) => Ok(id),
            NodeIdRead::Malformed => Err(invalid(meter, at, code)),
            NodeIdRead::Invalid(_) => {
                Err(invalid(meter, at, CheckedPackageRefusalCode::MalformedWire))
            }
        }
    };
    let items = array("modifies", meter)?;
    let mut modifies = meter.vec(items.clone().count())?;
    for (index, entry) in items.enumerate() {
        let at = root.key("modifies").index(index);
        let kind = entry
            .get("kind")
            .and_then(TermNode::as_str)
            .and_then(FrameEntryKind::from_wire);
        let members: &[&str] = match kind {
            Some(FrameEntryKind::Relationship) => &["kind", "declaration"],
            Some(FrameEntryKind::Field) => &["kind", "declaration", "name", "scope"],
            None => return Err(invalid(meter, at, code)),
        };
        if !entry.exact_members(members) {
            return Err(invalid(meter, at, code));
        }
        let declaration = read_id(entry.get("declaration"), meter, at)?;
        let value = if kind == Some(FrameEntryKind::Relationship) {
            ModifiesEntry::Relationship { declaration }
        } else {
            let name = entry
                .get("name")
                .and_then(TermNode::as_str)
                .filter(|name| is_identifier(name))
                .ok_or_else(|| invalid(meter, at, code))?;
            let scope = entry
                .get("scope")
                .and_then(TermNode::as_str)
                .and_then(FieldScope::from_wire)
                .ok_or_else(|| invalid(meter, at, code))?;
            ModifiesEntry::Field {
                declaration,
                name: meter.boxed_str(name)?,
                scope,
            }
        };
        modifies.push(value);
    }
    reject_repeat_in(&modifies, position, root.key("modifies"), meter)?;
    let mut read_keys = |key: &'static str| -> Result<Vec<CheckedNodeId>, ValidationFailure> {
        let items = array(key, meter)?;
        let mut keys = meter.vec(items.clone().count())?;
        for (index, item) in items.enumerate() {
            keys.push(read_id(Some(item), meter, root.key(key).index(index))?);
        }
        reject_repeat_in(&keys, position, root.key(key), meter)?;
        Ok(keys)
    };
    let creates = read_keys("creates")?;
    let deletes = read_keys("deletes")?;
    let items = array("deletes_scoped", meter)?;
    let mut deletes_scoped = meter.vec(items.clone().count())?;
    for (index, entry) in items.enumerate() {
        let at = root.key("deletes_scoped").index(index);
        let value = match entry.get("kind").and_then(TermNode::as_str) {
            Some("receiver") if entry.exact_members(&["kind"]) => DeletesScopedEntry::Receiver,
            Some("parameter") if entry.exact_members(&["kind", "name"]) => {
                let name = entry
                    .get("name")
                    .and_then(TermNode::as_str)
                    .filter(|name| is_identifier(name))
                    .ok_or_else(|| invalid(meter, at, code))?;
                DeletesScopedEntry::Parameter {
                    name: meter.boxed_str(name)?,
                }
            }
            _ => return Err(invalid(meter, at, code)),
        };
        deletes_scoped.push(value);
    }
    reject_repeat_in(&deletes_scoped, position, root.key("deletes_scoped"), meter)?;
    let work = 1_u64
        .saturating_add(modifies.len() as u64)
        .saturating_add(creates.len() as u64)
        .saturating_add(deletes.len() as u64)
        .saturating_add(deletes_scoped.len() as u64);
    Ok((
        FrameBody {
            modifies,
            creates,
            deletes,
            deletes_scoped,
        },
        work,
    ))
}

/// A paid sorted index finds the earliest repeated wire position without
/// an input-sized quadratic scan or an owned key copy.
fn reject_repeat_in<T: Ord>(
    entries: &[T],
    position: usize,
    at: Path<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut order = meter.vec(entries.len())?;
    order.extend(0..entries.len());
    order.sort_unstable_by(|&a, &b| entries[a].cmp(&entries[b]).then(a.cmp(&b)));
    if let Some(index) = order
        .windows(2)
        .filter(|pair| entries[pair[0]] == entries[pair[1]])
        .map(|pair| pair[1])
        .min()
    {
        return Err(stage_storage::refuse(
            meter,
            &Site::new(PackageReadStage::Graph, position, "state", at),
            Refusal::new(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at.index(index),
            ),
        ));
    }
    Ok(())
}

/// A repeated entry refuses at its second occurrence.
fn reject_repeat<T: Ord>(entries: &[T], at: &Trail<'_>) -> Result<(), ValidationFailure> {
    let mut seen = BTreeSet::new();
    match entries.iter().position(|entry| !seen.insert(entry)) {
        Some(index) => Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.index(index).pointer(),
        )),
        None => Ok(()),
    }
}

/// One node key, located at the entry that holds it.
fn read_node_key(value: &Value, at: &Trail<'_>) -> Result<CheckedNodeId, ValidationFailure> {
    let mut target = None;
    visit_reference(
        value,
        ReferenceMember::FrameEntry,
        false,
        at,
        &mut |id, _, _, _, _| {
            target = Some(id);
            Ok(())
        },
    )?;
    target.ok_or_else(|| {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    })
}

fn read_node_keys(
    values: &[Value],
    at: &Trail<'_>,
) -> Result<Vec<CheckedNodeId>, ValidationFailure> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| read_node_key(value, &at.index(index)))
        .collect()
}

/// One `FrameModifiesEntry` at `at`; any other shape refuses at the entry.
// Decodes a modifies entry's `kind`.
fn read_modifies_entry(entry: &Value, at: &Trail<'_>) -> Result<ModifiesEntry, ValidationFailure> {
    let invalid = || {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    };
    let Value::Object(object) = entry else {
        return Err(invalid());
    };
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let kind = FrameEntryKind::from_wire(kind).ok_or_else(invalid)?;
    let members: &[&str] = match kind {
        FrameEntryKind::Relationship => &["kind", "declaration"],
        FrameEntryKind::Field => &["kind", "declaration", "name", "scope"],
    };
    if !exact_members(object, members) {
        return Err(invalid());
    }
    let declaration = read_node_key(object.get("declaration").unwrap_or(&Value::Null), at)?;
    Ok(match kind {
        FrameEntryKind::Relationship => ModifiesEntry::Relationship { declaration },
        FrameEntryKind::Field => {
            let name = object
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| is_identifier(name))
                .ok_or_else(invalid)?;
            let scope = object
                .get("scope")
                .and_then(Value::as_str)
                .and_then(FieldScope::from_wire)
                .ok_or_else(invalid)?;
            ModifiesEntry::Field {
                declaration,
                name: name.into(),
                scope,
            }
        }
    })
}

/// One closed scoped-deletion entry at `at`.
fn read_deletes_scoped_entry(
    entry: &Value,
    at: &Trail<'_>,
) -> Result<DeletesScopedEntry, ValidationFailure> {
    let invalid = || {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    };
    let Value::Object(object) = entry else {
        return Err(invalid());
    };
    match object.get("kind").and_then(Value::as_str) {
        Some("receiver") if exact_members(object, &["kind"]) => Ok(DeletesScopedEntry::Receiver),
        Some("parameter") if exact_members(object, &["kind", "name"]) => {
            let name = object
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| is_identifier(name))
                .ok_or_else(invalid)?;
            Ok(DeletesScopedEntry::Parameter { name: name.into() })
        }
        _ => Err(invalid()),
    }
}

/// A frame body position, in FR-340's tie-break order.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Position {
    SemanticType,
    Modifies,
    Creates,
    Deletes,
    DeletesScoped,
}

impl Position {
    const fn wire_key(self) -> &'static str {
        match self {
            Self::SemanticType => "semantic_type",
            Self::Modifies => "modifies",
            Self::Creates => "creates",
            Self::Deletes => "deletes",
            Self::DeletesScoped => "deletes_scoped",
        }
    }
}

/// One meaning-join defect of a frame node; [`MeaningDefect::rank`] selects
/// the reported one.
#[derive(Clone, Debug)]
struct MeaningDefect<'a> {
    position: Position,
    node_id: &'a CheckedNodeId,
    name: &'a str,
    /// The entry's index in its member array; `None` for `semantic_type`.
    entry: Option<usize>,
    refusal: ModelRefusal,
}

impl MeaningDefect<'_> {
    /// Position, then entry declaration id, then field name.
    fn rank(&self) -> (Position, &CheckedNodeId, &str) {
        (self.position, self.node_id, self.name)
    }
}

/// The graph the frame and state steps read.
pub(super) struct StepGraph<'a, 'm, Body = super::CheckedJson> {
    pub(super) nodes: &'a [CheckedSemanticNodeV2<Body>],
    pub(super) kinds: &'a [CheckedNodeKind],
    pub(super) index: &'a GraphIndex<'a>,
    pub(super) owners: &'a ModelOwners<'m>,
}

impl<Body> StepGraph<'_, '_, Body> {
    pub(super) fn node(
        &self,
        id: &CheckedNodeId,
    ) -> Option<(&CheckedSemanticNodeV2<Body>, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    /// Every node of `kind`, as `(position, node)`, in ascending node-id order.
    pub(super) fn each(
        &self,
        kind: CheckedNodeKind,
    ) -> impl Iterator<Item = (usize, &CheckedSemanticNodeV2<Body>)> + '_ {
        self.index.values().filter_map(move |&position| {
            (self.kinds.get(position) == Some(&kind))
                .then(|| self.nodes.get(position).map(|node| (position, node)))
                .flatten()
        })
    }
}

/// Joins a reference `target` held by `holder` to a declared dependency of
/// the admitted kind: `missing-name` when it is not among `holder`'s
/// `dependencies` or names no node, `malformed-declaration` for another kind.
pub(super) fn join<'g, Body>(
    holder: &CheckedSemanticNodeV2<Body>,
    target: &CheckedNodeId,
    admits: impl Fn(CheckedNodeKind) -> bool,
    graph: &'g StepGraph<'_, '_, Body>,
) -> Result<(&'g CheckedSemanticNodeV2<Body>, CheckedNodeKind), ModelRefusal> {
    let (node, kind) = graph
        .node(target)
        .filter(|_| holder.dependencies.contains(target))
        .ok_or(ModelRefusal::missing_name())?;
    if admits(kind) {
        Ok((node, kind))
    } else {
        Err(ModelRefusal::malformed())
    }
}

/// Which node kinds an entry of one member may name.
fn creates_or_deletes(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::Model(ModelForm::ObjectType | ModelForm::Process)
    )
}

fn relationship_entry(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Relation(RelationForm::Relationship)
}

fn field_entry(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::Model(
            ModelForm::ObjectType
                | ModelForm::RecordValueType
                | ModelForm::Process
                | ModelForm::StateMachine
        )
    )
}

/// The frame step: every `state`/`frame` node, in ascending node-id
/// order, reporting the first one carrying a defect. `frames` holds each
/// frame node's key, position and body as [`read_frame_body`] read them.
pub(super) fn validate_frame_semantics<Body: super::model_members::EmptyAggregateBody>(
    mut frames: Vec<(&CheckedNodeId, usize, FrameBody)>,
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    frames.sort_unstable_by(|a, b| a.0.cmp(b.0));
    for (node_id, position, body) in &frames {
        meter.at_node(*position, "state");
        let Some(node) = nodes.get(*position) else {
            continue;
        };
        if let Some(failure) = frame_defect(node_id, node, *position, body, &graph, meter)? {
            return Err(failure);
        }
    }
    Ok(())
}

/// The one refusal FR-340 selects for a frame node, or `None`.
fn frame_defect<Body: super::model_members::EmptyAggregateBody>(
    frame_id: &CheckedNodeId,
    frame: &CheckedSemanticNodeV2<Body>,
    frame_position: usize,
    body: &FrameBody,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let mut defect: Option<MeaningDefect<'_>> = None;
    fn retain<'a>(defect: &mut Option<MeaningDefect<'a>>, candidate: MeaningDefect<'a>) {
        if defect
            .as_ref()
            .is_none_or(|held| candidate.rank() < held.rank())
        {
            *defect = Some(candidate);
        }
    }
    let receiver_scoped = matches!(
        graph.node(&frame.semantic_type),
        Some((
            _,
            CheckedNodeKind::Model(ModelForm::Process | ModelForm::StateMachine)
        ))
    );
    match graph.node(&frame.semantic_type) {
        Some((
            _,
            CheckedNodeKind::Model(
                ModelForm::ObjectType | ModelForm::Process | ModelForm::StateMachine,
            ),
        )) => {}
        found => retain(
            &mut defect,
            MeaningDefect {
                position: Position::SemanticType,
                node_id: &frame.semantic_type,
                name: "",
                entry: None,
                refusal: if found.is_some() {
                    ModelRefusal::malformed()
                } else {
                    ModelRefusal::missing_name()
                },
            },
        ),
    }
    for (entry_index, entry) in body.modifies.iter().enumerate() {
        let admits = match entry {
            ModifiesEntry::Relationship { .. } => relationship_entry,
            ModifiesEntry::Field { .. } => field_entry,
        };
        let refusal = match join(frame, entry.declaration(), admits, graph) {
            Err(refusal) => Some(refusal),
            Ok((node, kind)) => match entry {
                ModifiesEntry::Relationship { .. } => None,
                ModifiesEntry::Field { name, scope, .. } => {
                    match resolve_field(node, kind, name, graph, meter)? {
                        Some(refusal) => Some(refusal),
                        None if receiver_scoped && *scope == FieldScope::Receiver => {
                            Some(ModelRefusal::malformed())
                        }
                        None => None,
                    }
                }
            },
        };
        if let Some(refusal) = refusal {
            retain(
                &mut defect,
                MeaningDefect {
                    position: Position::Modifies,
                    node_id: entry.declaration(),
                    name: entry.name(),
                    entry: Some(entry_index),
                    refusal,
                },
            );
        }
    }
    for (position, entries) in [
        (Position::Creates, &body.creates),
        (Position::Deletes, &body.deletes),
    ] {
        for (entry_index, entry) in entries.iter().enumerate() {
            if let Err(refusal) = join(frame, entry, creates_or_deletes, graph) {
                retain(
                    &mut defect,
                    MeaningDefect {
                        position,
                        node_id: entry,
                        name: "",
                        entry: Some(entry_index),
                        refusal,
                    },
                );
            }
        }
    }
    if receiver_scoped && body.deletes_scoped.contains(&DeletesScopedEntry::Receiver) {
        retain(
            &mut defect,
            MeaningDefect {
                position: Position::DeletesScoped,
                node_id: frame_id,
                name: "",
                entry: None,
                refusal: ModelRefusal::malformed(),
            },
        );
    }
    if let Some(defect) = defect {
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(frame_position);
        let path = match defect.entry {
            _ if defect.position == Position::DeletesScoped => at,
            None => at.key(defect.position.wire_key()),
            Some(entry) => at.key("body").key(defect.position.wire_key()).index(entry),
        };
        return Ok(Some(stage_storage::refuse(
            meter,
            &Site::new(PackageReadStage::Graph, frame_position, "frame", path),
            Refusal::new(defect.refusal.code, path)
                .because(defect.refusal.cause)
                .at_node(defect.node_id),
        )));
    }
    let mut scopes = meter.vec(
        body.modifies
            .iter()
            .filter(|entry| matches!(entry, ModifiesEntry::Field { .. }))
            .count(),
    )?;
    scopes.extend(
        body.modifies
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                ModifiesEntry::Field {
                    declaration,
                    name,
                    scope,
                } => Some((declaration, name.as_ref(), index, *scope)),
                ModifiesEntry::Relationship { .. } => None,
            }),
    );
    scopes.sort_unstable_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)));
    let conflict = scopes.windows(2).find_map(|pair| {
        let (declaration, name, earlier_index, earlier_scope) = pair[0];
        let (held, held_name, entry_index, scope) = pair[1];
        (declaration == held && name == held_name && earlier_scope != scope).then_some((
            declaration,
            name,
            if scope == FieldScope::Receiver {
                entry_index
            } else {
                earlier_index
            },
        ))
    });
    if let Some((declaration, _, receiver_index)) = conflict {
        let path = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(frame_position)
            .key("body")
            .key("modifies")
            .index(receiver_index);
        return Ok(Some(stage_storage::refuse(
            meter,
            &Site::new(PackageReadStage::Graph, frame_position, "frame", path),
            Refusal::new(CheckedPackageRefusalCode::InvalidModelBinding, path)
                .because(CheckedPackageRefusalCause::ConflictingBinding)
                .at_node(declaration),
        )));
    }
    let ascending = |keys: &[CheckedNodeId]| keys.windows(2).all(|pair| pair[0] < pair[1]);
    let modifies_ordered = body
        .modifies
        .windows(2)
        .all(|pair| pair[0].order_key() < pair[1].order_key());
    let scoped_ordered = body.deletes_scoped.windows(2).all(|pair| pair[0] < pair[1]);
    let in_order =
        modifies_ordered && scoped_ordered && ascending(&body.creates) && ascending(&body.deletes);
    if in_order {
        Ok(None)
    } else {
        let path = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(frame_position)
            .key("body");
        Ok(Some(stage_storage::refuse(
            meter,
            &Site::new(PackageReadStage::Graph, frame_position, "frame", path),
            Refusal::new(CheckedPackageRefusalCode::InvalidSemanticGraph, path).at_node(frame_id),
        )))
    }
}

/// A field entry's name among the fields of its declaring node.
///
/// A selected object or record declaration resolves through its effective
/// fields. Anonymous and source-declared records carry no selected fields.
fn resolve_field<Body: super::model_members::EmptyAggregateBody>(
    declaring: &CheckedSemanticNodeV2<Body>,
    kind: CheckedNodeKind,
    name: &str,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<Option<ModelRefusal>, ValidationFailure> {
    if (kind == CheckedNodeKind::Model(ModelForm::RecordValueType)
        && !matches!(
            declaring.owner.as_ref(),
            Some(super::CheckedNodeOwner::Model { .. })
        ))
        || !graph
            .owners
            .is_model_declaration_node(declaring, kind.tag())
    {
        return Ok(None);
    }
    Ok(
        match graph
            .owners
            .resolve_member(declaring, MemberKind::Field, name, meter)?
        {
            Ok(_) => None,
            Err(refusal) if refusal == ModelRefusal::ineligible() => {
                Some(ModelRefusal::missing_name())
            }
            Err(refusal) => Some(refusal),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{creates_or_deletes, field_entry, is_identifier, relationship_entry};
    use crate::checked_package::v2::{CheckedNodeKind, ModelForm, RelationForm};
    use ix_trace_rs::trace;

    fn admitted(admits: fn(CheckedNodeKind) -> bool) -> Vec<CheckedNodeKind> {
        CheckedNodeKind::all()
            .into_iter()
            .filter(|kind| admits(*kind))
            .collect()
    }

    /// A `creates` or `deletes` entry names exactly a `model`/`object_type`
    /// or `model`/`process` node, over every kind the closed vocabularies
    /// produce.
    #[trace("TC-053", "FR-038-AC-12")]
    #[test]
    fn tc_053_creates_and_deletes_admit_exactly_object_types_and_processes() {
        assert_eq!(
            admitted(creates_or_deletes),
            [
                CheckedNodeKind::Model(ModelForm::ObjectType),
                CheckedNodeKind::Model(ModelForm::Process),
            ]
        );
    }

    /// A relationship entry names exactly a `relation`/`relationship` node
    /// and a field entry exactly a field-bearing model declaration; no
    /// systems form is a declaring node.
    #[trace("TC-056", "FR-040-AC-2")]
    #[test]
    fn tc_056_modifies_entries_admit_exactly_their_declaring_kinds() {
        assert_eq!(
            admitted(relationship_entry),
            [CheckedNodeKind::Relation(RelationForm::Relationship)]
        );
        assert_eq!(
            admitted(field_entry),
            [
                CheckedNodeKind::Model(ModelForm::ObjectType),
                CheckedNodeKind::Model(ModelForm::RecordValueType),
                CheckedNodeKind::Model(ModelForm::StateMachine),
                CheckedNodeKind::Model(ModelForm::Process),
            ]
        );
    }

    #[trace("TC-056")]
    #[test]
    fn tc_056_field_names_are_identifiers() {
        for name in ["total", "_x", "a1", "A_b"] {
            assert!(is_identifier(name), "{name}");
        }
        for name in ["", "1a", "a-b", "a b", "é"] {
            assert!(!is_identifier(name), "{name}");
        }
    }
}

#[cfg(test)]
mod document_storage_tests {
    use super::super::{CheckedJson, NativeJoinBudget};
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn document(duplicate: bool) -> CheckedJson {
        let id = json!({"package":"pkg", "path":[{"operand":0}], "ordinal":0});
        let creates = if duplicate {
            json!([id, id])
        } else {
            json!([id])
        };
        serde_json::from_value(json!({"term":"frame", "modifies":[{"kind":"field", "declaration":id, "name":"field", "scope":"receiver"}], "creates":creates, "deletes":[], "deletes_scoped":[{"kind":"parameter", "name":"argument"}]})).unwrap()
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn frame_document_and_retained_term_pay_exact_blocks_before_allocation() {
        let body = document(false);
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(7, "frame");
                result = Some(
                    read_frame_body(
                        super::super::term_view::TermNode::of(&body),
                        7,
                        &[42],
                        &mut meter,
                    )
                    .and_then(|(frame, work)| {
                        frame
                            .checked_term(&mut meter)
                            .map(|term| (frame, term, work))
                    }),
                );
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        let (frame, term, work) = result.unwrap();
        assert_eq!(work, 4);
        assert_eq!(
            frame.creates[0].path,
            vec![crate::checked_package::shared::CheckedNodePathSegment::Index(42)]
        );
        let view = FrameView::of(&term).unwrap();
        assert_eq!(view.work(), work);
        assert!(
            matches!(&view.modifies[0], CheckedFrameModification::Field { name, scope: CheckedFieldScope::Receiver, .. } if name.as_ref() == "field")
        );
        assert!(
            matches!(&view.deletes_scoped[0], CheckedScopedDeletion::Parameter { name } if name.as_ref() == "argument")
        );
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required);
        assert!(run(required).0.is_ok());
        assert!(matches!(
            run(required - 1).0,
            Err(ValidationFailure::Budget(_))
        ));
        assert_eq!(run(0).1, 0);
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn frame_duplicate_keeps_second_entry_fault_and_prepays_diagnostic() {
        let body = document(true);
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(7, "frame");
                result = Some(read_frame_body(
                    super::super::term_view::TermNode::of(&body),
                    7,
                    &[42],
                    &mut meter,
                ));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("duplicate must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            "/semantic_graph/nodes/7/body/creates/1"
        );
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required);
        assert!(matches!(
            run(required).0,
            Err(ValidationFailure::Refused(_))
        ));
        assert!(matches!(
            run(required - 1).0,
            Err(ValidationFailure::Budget(_))
        ));
    }
}
