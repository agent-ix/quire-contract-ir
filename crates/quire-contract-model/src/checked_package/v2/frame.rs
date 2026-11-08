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
use super::{
    BodyTerm, CheckedFieldScope, CheckedFrameModification, CheckedNodeKind, CheckedScopedDeletion,
    CheckedTerm, FrameEntryKind, ModelForm, RawSemanticNodeV2 as CheckedSemanticNodeV2,
    RelationForm, TermNode, WorkMeter,
};
use crate::checked_package::common::{node_pointer, Trail, ValidationFailure};
use crate::checked_package::shared::CheckedPackageRefusalCause;
use crate::checked_package::shared::{CheckedNodeId, CheckedPackageRefusalCode};
use std::collections::BTreeMap;

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
    /// Copies the already validated frame into its public typed term, charging
    /// every retained vector and owned value before requesting the allocation.
    pub(super) fn checked_term(
        &self,
        meter: &mut WorkMeter,
    ) -> Result<CheckedTerm, ValidationFailure> {
        let mut modifies = meter.vec(self.modifies.len())?;
        for entry in &self.modifies {
            let copied = match entry {
                ModifiesEntry::Relationship { declaration } => {
                    CheckedFrameModification::Relationship {
                        declaration: meter.clone_id(declaration)?,
                    }
                }
                ModifiesEntry::Field {
                    declaration,
                    name,
                    scope,
                } => {
                    let declaration = meter.clone_id(declaration)?;
                    let mut copied_name = meter.string(name.len())?;
                    copied_name.push_str(name);
                    CheckedFrameModification::Field {
                        declaration,
                        name: copied_name,
                        scope: match scope {
                            FieldScope::EveryObject => CheckedFieldScope::EveryObject,
                            FieldScope::Receiver => CheckedFieldScope::Receiver,
                        },
                    }
                }
            };
            meter.push(&mut modifies, copied)?;
        }
        let mut deletes_scoped = meter.vec(self.deletes_scoped.len())?;
        for entry in &self.deletes_scoped {
            let copied = match entry {
                DeletesScopedEntry::Receiver => CheckedScopedDeletion::Receiver,
                DeletesScopedEntry::Parameter { name } => {
                    let mut copied_name = meter.string(name.len())?;
                    copied_name.push_str(name);
                    CheckedScopedDeletion::Parameter { name: copied_name }
                }
            };
            meter.push(&mut deletes_scoped, copied)?;
        }
        let mut creates = meter.vec(self.creates.len())?;
        for id in &self.creates {
            let copied = meter.clone_id(id)?;
            meter.push(&mut creates, copied)?;
        }
        let mut deletes = meter.vec(self.deletes.len())?;
        for id in &self.deletes {
            let copied = meter.clone_id(id)?;
            meter.push(&mut deletes, copied)?;
        }
        Ok(CheckedTerm::Frame {
            modifies,
            creates,
            deletes,
            deletes_scoped,
        })
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

/// Reads a `state`/`frame` node's body at `at`, returning it and its work:
/// one unit for the body and one per entry.
pub(super) fn read_frame_body(
    body: TermNode<'_>,
    row: &[u64],
    meter: &mut WorkMeter,
    at: &Trail<'_>,
) -> Result<(FrameBody, u64), ValidationFailure> {
    let invalid = |position: &Trail<'_>| {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            position.pointer(),
        )
    };
    if !body.is_object() {
        return Err(invalid(at));
    }
    if !body.exact_members(&["term", "modifies", "creates", "deletes", "deletes_scoped"]) {
        return Err(invalid(at));
    }
    if body.kind() != Some(BodyTerm::Frame) {
        return Err(invalid(&at.key("term")));
    }
    let entries =
        |key: &str| -> Result<crate::checked_package::v2::TermItems<'_>, ValidationFailure> {
            body.get(key)
                .and_then(TermNode::items)
                .ok_or_else(|| invalid(&at.key(key)))
        };
    let modifies_at = at.key("modifies");
    let modifies_items = entries("modifies")?;
    let mut modifies = meter.vec(modifies_items.len())?;
    for (index, entry) in modifies_items.enumerate() {
        let parsed = read_modifies_entry(entry, row, meter, &modifies_at.index(index))?;
        meter.push(&mut modifies, parsed)?;
    }
    reject_repeat(&modifies, &modifies_at)?;
    let creates_at = at.key("creates");
    let creates = read_node_keys(entries("creates")?, row, meter, &creates_at)?;
    reject_repeat(&creates, &creates_at)?;
    let deletes_at = at.key("deletes");
    let deletes = read_node_keys(entries("deletes")?, row, meter, &deletes_at)?;
    reject_repeat(&deletes, &deletes_at)?;
    let deletes_scoped_at = at.key("deletes_scoped");
    let deletes_items = entries("deletes_scoped")?;
    let mut deletes_scoped = meter.vec(deletes_items.len())?;
    for (index, entry) in deletes_items.enumerate() {
        let parsed = read_deletes_scoped_entry(entry, meter, &deletes_scoped_at.index(index))?;
        meter.push(&mut deletes_scoped, parsed)?;
    }
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

/// A repeated entry refuses at its second occurrence.
fn reject_repeat<T: Ord>(entries: &[T], at: &Trail<'_>) -> Result<(), ValidationFailure> {
    match entries
        .iter()
        .enumerate()
        .find_map(|(index, entry)| entries.get(..index)?.contains(entry).then_some(index))
    {
        Some(index) => Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.index(index).pointer(),
        )),
        None => Ok(()),
    }
}

/// One node key, located at the entry that holds it.
fn read_node_key(
    value: TermNode<'_>,
    row: &[u64],
    meter: &mut WorkMeter,
    at: &Trail<'_>,
) -> Result<CheckedNodeId, ValidationFailure> {
    meter.read_id(value, row)?.valid().ok_or_else(|| {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    })
}

fn read_node_keys(
    values: crate::checked_package::v2::TermItems<'_>,
    row: &[u64],
    meter: &mut WorkMeter,
    at: &Trail<'_>,
) -> Result<Vec<CheckedNodeId>, ValidationFailure> {
    let count = values.len();
    let mut keys = meter.vec(count)?;
    for (index, value) in values.enumerate() {
        let key = read_node_key(value, row, meter, &at.index(index))?;
        meter.push(&mut keys, key)?;
    }
    Ok(keys)
}

/// One `FrameModifiesEntry` at `at`; any other shape refuses at the entry.
// Decodes a modifies entry's `kind`.
fn read_modifies_entry(
    entry: TermNode<'_>,
    row: &[u64],
    meter: &mut WorkMeter,
    at: &Trail<'_>,
) -> Result<ModifiesEntry, ValidationFailure> {
    let invalid = || {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    };
    if !entry.is_object() {
        return Err(invalid());
    }
    let kind = entry
        .get("kind")
        .and_then(TermNode::as_str)
        .unwrap_or_default();
    let kind = FrameEntryKind::from_wire(kind).ok_or_else(invalid)?;
    let members: &[&str] = match kind {
        FrameEntryKind::Relationship => &["kind", "declaration"],
        FrameEntryKind::Field => &["kind", "declaration", "name", "scope"],
    };
    if !entry.exact_members(members) {
        return Err(invalid());
    }
    let declaration = read_node_key(
        entry.get("declaration").ok_or_else(invalid)?,
        row,
        meter,
        at,
    )?;
    Ok(match kind {
        FrameEntryKind::Relationship => ModifiesEntry::Relationship { declaration },
        FrameEntryKind::Field => {
            let name = entry
                .get("name")
                .and_then(TermNode::as_str)
                .filter(|name| is_identifier(name))
                .ok_or_else(invalid)?;
            let scope = entry
                .get("scope")
                .and_then(TermNode::as_str)
                .and_then(FieldScope::from_wire)
                .ok_or_else(invalid)?;
            ModifiesEntry::Field {
                declaration,
                name: meter.boxed_str(name)?,
                scope,
            }
        }
    })
}

/// One closed scoped-deletion entry at `at`.
fn read_deletes_scoped_entry(
    entry: TermNode<'_>,
    meter: &mut WorkMeter,
    at: &Trail<'_>,
) -> Result<DeletesScopedEntry, ValidationFailure> {
    let invalid = || {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    };
    if !entry.is_object() {
        return Err(invalid());
    }
    match entry.get("kind").and_then(TermNode::as_str) {
        Some("receiver") if entry.exact_members(&["kind"]) => Ok(DeletesScopedEntry::Receiver),
        Some("parameter") if entry.exact_members(&["kind", "name"]) => {
            let name = entry
                .get("name")
                .and_then(TermNode::as_str)
                .filter(|name| is_identifier(name))
                .ok_or_else(invalid)?;
            Ok(DeletesScopedEntry::Parameter {
                name: meter.boxed_str(name)?,
            })
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
struct MeaningDefect {
    position: Position,
    node_id: CheckedNodeId,
    name: Box<str>,
    /// The entry's index in its member array; `None` for `semantic_type`.
    entry: Option<usize>,
    refusal: ModelRefusal,
}

impl MeaningDefect {
    /// Position, then entry declaration id, then field name.
    fn rank(&self) -> (Position, &CheckedNodeId, &str) {
        (self.position, &self.node_id, &self.name)
    }
}

/// The graph the frame and state steps read.
pub(super) struct StepGraph<'a, 'm, Body = super::CheckedJson> {
    pub(super) nodes: &'a [super::CheckedSemanticNodeV2<Body>],
    pub(super) kinds: &'a [CheckedNodeKind],
    pub(super) index: &'a GraphIndex<'a>,
    pub(super) owners: &'a ModelOwners<'m>,
}

impl<Body> StepGraph<'_, '_, Body> {
    pub(super) fn node(
        &self,
        id: &CheckedNodeId,
    ) -> Option<(&super::CheckedSemanticNodeV2<Body>, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    /// Every node of `kind`, as `(position, node)`, in ascending node-id order.
    pub(super) fn each(
        &self,
        kind: CheckedNodeKind,
    ) -> impl Iterator<Item = (usize, &super::CheckedSemanticNodeV2<Body>)> + '_ {
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
    holder: &super::CheckedSemanticNodeV2<Body>,
    target: &CheckedNodeId,
    admits: impl Fn(CheckedNodeKind) -> bool,
    graph: &'g StepGraph<'_, '_, Body>,
) -> Result<(&'g super::CheckedSemanticNodeV2<Body>, CheckedNodeKind), ModelRefusal> {
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
pub(super) fn validate_frame_semantics(
    mut frames: Vec<(&CheckedNodeId, usize, FrameBody)>,
    nodes: &[CheckedSemanticNodeV2],
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
    frames.sort_by(|a, b| a.0.cmp(b.0));
    for (node_id, position, body) in &frames {
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
fn frame_defect(
    frame_id: &CheckedNodeId,
    frame: &CheckedSemanticNodeV2,
    frame_position: usize,
    body: &FrameBody,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let mut defects = Vec::new();
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
        found => defects.push(MeaningDefect {
            position: Position::SemanticType,
            node_id: frame.semantic_type.clone(),
            name: "".into(),
            entry: None,
            refusal: if found.is_some() {
                ModelRefusal::malformed()
            } else {
                ModelRefusal::missing_name()
            },
        }),
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
            defects.push(MeaningDefect {
                position: Position::Modifies,
                node_id: entry.declaration().clone(),
                name: entry.name().into(),
                entry: Some(entry_index),
                refusal,
            });
        }
    }
    for (position, entries) in [
        (Position::Creates, &body.creates),
        (Position::Deletes, &body.deletes),
    ] {
        for (entry_index, entry) in entries.iter().enumerate() {
            if let Err(refusal) = join(frame, entry, creates_or_deletes, graph) {
                defects.push(MeaningDefect {
                    position,
                    node_id: entry.clone(),
                    name: "".into(),
                    entry: Some(entry_index),
                    refusal,
                });
            }
        }
    }
    if receiver_scoped && body.deletes_scoped.contains(&DeletesScopedEntry::Receiver) {
        defects.push(MeaningDefect {
            position: Position::DeletesScoped,
            node_id: frame_id.clone(),
            name: "".into(),
            entry: None,
            refusal: ModelRefusal::malformed(),
        });
    }
    if let Some(defect) = defects.into_iter().min_by(|a, b| a.rank().cmp(&b.rank())) {
        let at = node_pointer(frame_position);
        let path = match defect.entry {
            _ if defect.position == Position::DeletesScoped => at,
            None => at.key(defect.position.wire_key()),
            Some(entry) => at.key("body").key(defect.position.wire_key()).index(entry),
        };
        return Ok(Some(ValidationFailure::refused_at(
            defect.refusal.code,
            path,
            Some(defect.refusal.cause),
            defect.node_id,
        )));
    }
    let mut scopes = BTreeMap::new();
    let mut conflict = None;
    for (entry_index, entry) in body.modifies.iter().enumerate() {
        if let ModifiesEntry::Field {
            declaration,
            name,
            scope,
        } = entry
        {
            let key = (declaration, name);
            if let Some((earlier_scope, earlier_index)) = scopes.insert(key, (*scope, entry_index))
            {
                if earlier_scope != *scope {
                    let receiver_index = if *scope == FieldScope::Receiver {
                        entry_index
                    } else {
                        earlier_index
                    };
                    let candidate = (declaration, name.as_ref(), receiver_index);
                    if conflict
                        .as_ref()
                        .is_none_or(|best: &(&CheckedNodeId, &str, usize)| {
                            (candidate.0, candidate.1) < (best.0, best.1)
                        })
                    {
                        conflict = Some(candidate);
                    }
                }
            }
        }
    }
    if let Some((declaration, _, receiver_index)) = conflict {
        return Ok(Some(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            node_pointer(frame_position)
                .key("body")
                .key("modifies")
                .index(receiver_index),
            Some(CheckedPackageRefusalCause::ConflictingBinding),
            declaration.clone(),
        )));
    }
    let ascending = |keys: Vec<&CheckedNodeId>| keys.windows(2).all(|pair| pair[0] < pair[1]);
    let modifies_ordered = body
        .modifies
        .windows(2)
        .all(|pair| pair[0].order_key() < pair[1].order_key());
    let scoped_ordered = body.deletes_scoped.windows(2).all(|pair| pair[0] < pair[1]);
    let in_order = modifies_ordered
        && scoped_ordered
        && ascending(body.creates.iter().collect())
        && ascending(body.deletes.iter().collect());
    if in_order {
        Ok(None)
    } else {
        Ok(Some(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(frame_position).key("body"),
            None,
            frame_id.clone(),
        )))
    }
}

/// A field entry's name among the fields of its declaring node.
///
/// A selected object or record declaration resolves through its effective
/// fields. Anonymous and source-declared records carry no selected fields.
fn resolve_field(
    declaring: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    name: &str,
    graph: &StepGraph<'_, '_>,
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

    fn document() -> CheckedJson {
        let id = json!({"package":"pkg", "path":[{"operand":0}], "ordinal":0});
        serde_json::from_value(json!({"term":"frame", "modifies":[{"kind":"field", "declaration":id, "name":"field", "scope":"receiver"}], "creates":[id], "deletes":[], "deletes_scoped":[{"kind":"parameter", "name":"argument"}]})).unwrap()
    }

    #[trace("TC-472", "FR-038-AC-504", "FR-038-AC-511")]
    #[test]
    fn frame_document_and_retained_term_pay_exact_blocks_before_allocation() {
        let body = document();
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(7, "state");
                result = Some(
                    read_frame_body(TermNode::of(&body), &[42], &mut meter, &Trail::Base(&[]))
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
        assert_eq!(
            term,
            frame
                .into_checked_term(&mut WorkMeter::new(u64::MAX))
                .unwrap()
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

    #[trace("TC-472", "FR-038-AC-504", "FR-038-AC-511")]
    #[test]
    fn borrowed_frame_copy_keeps_its_source_and_stops_before_unpaid_storage() {
        let body = document();
        let (frame, _) = read_frame_body(
            TermNode::of(&body),
            &[42],
            &mut WorkMeter::new(u64::MAX),
            &Trail::Base(&[]),
        )
        .unwrap();
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(7, "state");
                result = Some(frame.checked_term(&mut meter));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        let expected = result.unwrap();
        let required = budget.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required);
        assert_eq!(run(required).0.unwrap(), expected);
        let (result, allocated, budget) = run(required - 1);
        assert!(matches!(result, Err(ValidationFailure::Budget(_))));
        assert_eq!(allocated, budget.read_storage_consumed());
        assert_eq!(
            budget.first_unpaid_read_storage(),
            Some("argument".len() as u128)
        );
        let locus = budget.first_unpaid_locus().unwrap();
        assert_eq!(
            locus.source,
            super::super::NativeJoinLocusSource::PackageRead(
                super::super::PackageReadStage::Graph,
                7
            )
        );
        assert_eq!(locus.form, "state");
        assert_eq!(run(u64::MAX).0.unwrap(), expected);
    }
}
