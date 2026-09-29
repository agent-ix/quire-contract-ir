//! QSpec FR-340 frame bodies: the closed `{term: "frame", modifies, creates,
//! deletes}` body of a `state`/`frame` node, and the frame step of the
//! reader order.
//!
//! [`read_frame_body`] is the shape check the per-node body loop runs: each
//! member an array of distinct entries, `creates` and `deletes` of node keys
//! and `modifies` of `FrameModifiesEntry` values, `{kind: "relationship",
//! declaration}` or `{kind: "field", declaration, name}`. A shape defect
//! refuses as `invalid_semantic_graph` at the entry (a node key outside the
//! node domain as `digest_domain_mismatch`), a repeated entry at its second
//! occurrence.
//!
//! [`validate_frame_semantics`] is the frame step, run after every
//! graph-shape, stale-key and declaration refusal. For each frame node, in
//! ascending node-id digest order, it collects every meaning-join defect:
//!
//! - the frame's `semantic_type` names no node (`missing-name`) or a node
//!   that is not `model`/`object_type` (`malformed-declaration`);
//! - an entry whose node key is not among the frame's `dependencies` or
//!   names no node (`missing-name`), or names a node its kind or member
//!   cannot carry (`malformed-declaration`);
//! - a field entry whose name does not resolve among the fields of its
//!   declaring node (FR-322 "Model-owned members" steps 2 and 3): the owner
//!   refusal, `missing-name` for no field of that name, `ambiguous-name` for
//!   two.
//!
//! A field entry's name resolves through the selected domain package only
//! when its declaring node is a model declaration node (the scope FR-038's
//! "Model-owned members" gives the resolution). A declaring node that carries
//! its own `declaration` is declared by this package's source, names no
//! member in its body, and is not resolved against any domain package.
//!
//! The reported defect is the least by (position, entry declaration digest,
//! field name), positions ordered `semantic_type`, `modifies`, `creates`,
//! `deletes`; with no meaning-join defect, a member not strictly ascending by
//! its order key refuses as `invalid_semantic_graph` at the frame body,
//! located at the frame node. The first frame carrying a defect is reported.

use super::identity::is_identifier;
use super::model_members::{MemberKind, ModelOwners, ModelRefusal};
use super::{
    BodyTerm, CheckedNodeKind, CheckedSemanticNodeV2, FrameEntryKind, ModelForm, RelationForm,
    WorkMeter,
};
use crate::checked_package::common::{
    body_term, exact_members, node_pointer, visit_reference, ReferenceMember, Trail,
    ValidationFailure,
};
use crate::checked_package::shared::{CheckedNodeId, CheckedPackageRefusalCode};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// One `modifies` entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ModifiesEntry {
    /// `{kind: "relationship", declaration}`.
    Relationship { declaration: CheckedNodeId },
    /// `{kind: "field", declaration, name}`.
    Field {
        declaration: CheckedNodeId,
        name: Box<str>,
    },
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

    /// FR-340's order key: the declaration digest, then the field name.
    fn order_key(&self) -> (&str, &str) {
        (&self.declaration().digest, self.name())
    }
}

/// A frame body as the wire carries it, entries in wire order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct FrameBody {
    pub(super) modifies: Vec<ModifiesEntry>,
    pub(super) creates: Vec<CheckedNodeId>,
    pub(super) deletes: Vec<CheckedNodeId>,
}

/// Reads a `state`/`frame` node's body at `at`, returning it and its work:
/// one unit for the body and one per entry.
pub(super) fn read_frame_body(
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
    if !exact_members(object, &["term", "modifies", "creates", "deletes"]) {
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
    let count = |len: usize| u64::try_from(len).unwrap_or(u64::MAX);
    let work = 1_u64
        .saturating_add(count(modifies.len()))
        .saturating_add(count(creates.len()))
        .saturating_add(count(deletes.len()));
    Ok((
        FrameBody {
            modifies,
            creates,
            deletes,
        },
        work,
    ))
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
        &mut |id, _, _| {
            target = Some(id.clone());
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
// string-edge: decodes a modifies entry's `kind`.
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
        FrameEntryKind::Field => &["kind", "declaration", "name"],
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
            ModifiesEntry::Field {
                declaration,
                name: name.into(),
            }
        }
    })
}

/// A frame body position, in FR-340's tie-break order.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Position {
    SemanticType,
    Modifies,
    Creates,
    Deletes,
}

impl Position {
    const fn wire_key(self) -> &'static str {
        match self {
            Self::SemanticType => "semantic_type",
            Self::Modifies => "modifies",
            Self::Creates => "creates",
            Self::Deletes => "deletes",
        }
    }
}

/// One meaning-join defect of a frame node; [`MeaningDefect::rank`] selects
/// the reported one.
#[derive(Clone, Debug)]
struct MeaningDefect {
    position: Position,
    digest: Box<str>,
    name: Box<str>,
    /// The entry's index in its member array; `None` for `semantic_type`.
    entry: Option<usize>,
    refusal: ModelRefusal,
}

impl MeaningDefect {
    /// Position, then entry declaration digest, then field name.
    fn rank(&self) -> (Position, &str, &str) {
        (self.position, &self.digest, &self.name)
    }
}

/// The graph the frame and state steps read.
pub(super) struct StepGraph<'a, 'm> {
    pub(super) nodes: &'a [CheckedSemanticNodeV2],
    pub(super) kinds: &'a [CheckedNodeKind],
    pub(super) index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    pub(super) owners: &'a ModelOwners<'m>,
}

impl StepGraph<'_, '_> {
    pub(super) fn node(
        &self,
        id: &CheckedNodeId,
    ) -> Option<(&CheckedSemanticNodeV2, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    /// Every node of `kind`, as `(position, node)`, in ascending node-id
    /// digest order.
    pub(super) fn each(
        &self,
        kind: CheckedNodeKind,
    ) -> impl Iterator<Item = (usize, &CheckedSemanticNodeV2)> + '_ {
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
pub(super) fn join<'g>(
    holder: &CheckedSemanticNodeV2,
    target: &CheckedNodeId,
    admits: impl Fn(CheckedNodeKind) -> bool,
    graph: &'g StepGraph<'_, '_>,
) -> Result<(&'g CheckedSemanticNodeV2, CheckedNodeKind), ModelRefusal> {
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
        CheckedNodeKind::Model(ModelForm::ObjectType | ModelForm::RecordValueType)
    )
}

/// The frame step: every `state`/`frame` node, in ascending node-id digest
/// order, reporting the first one carrying a defect. `frames` holds each
/// frame node's key, position and body as [`read_frame_body`] read them.
pub(super) fn validate_frame_semantics(
    mut frames: Vec<(&CheckedNodeId, usize, FrameBody)>,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
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
    match graph.node(&frame.semantic_type) {
        Some((_, CheckedNodeKind::Model(ModelForm::ObjectType))) => {}
        found => defects.push(MeaningDefect {
            position: Position::SemanticType,
            digest: frame.semantic_type.digest.clone(),
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
                ModifiesEntry::Field { name, .. } => resolve_field(node, kind, name, graph, meter)?,
            },
        };
        if let Some(refusal) = refusal {
            defects.push(MeaningDefect {
                position: Position::Modifies,
                digest: entry.declaration().digest.clone(),
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
                    digest: entry.digest.clone(),
                    name: "".into(),
                    entry: Some(entry_index),
                    refusal,
                });
            }
        }
    }
    if let Some(defect) = defects.into_iter().min_by(|a, b| a.rank().cmp(&b.rank())) {
        let at = node_pointer(frame_position);
        let path = match defect.entry {
            None => at.key(defect.position.wire_key()),
            Some(entry) => at.key("body").key(defect.position.wire_key()).index(entry),
        };
        return Ok(Some(ValidationFailure::refused_at(
            defect.refusal.code,
            path,
            Some(defect.refusal.cause),
            CheckedNodeId {
                domain: frame.semantic_type.domain.clone(),
                digest: defect.digest,
            },
        )));
    }
    let ascending = |keys: Vec<(&str, &str)>| keys.windows(2).all(|pair| pair[0] < pair[1]);
    let in_order = ascending(body.modifies.iter().map(ModifiesEntry::order_key).collect())
        && ascending(body.creates.iter().map(|id| (&*id.digest, "")).collect())
        && ascending(body.deletes.iter().map(|id| (&*id.digest, "")).collect());
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
/// A `model`/`record_value_type` declaring node is never resolved: QSpec's
/// `ModelDeclarationNode` has no `record_value_type` form, so no domain
/// package can own one, and its field name is admitted unresolved.
fn resolve_field(
    declaring: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    name: &str,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<Option<ModelRefusal>, ValidationFailure> {
    if kind == CheckedNodeKind::Model(ModelForm::RecordValueType)
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

    fn admitted(admits: fn(CheckedNodeKind) -> bool) -> Vec<CheckedNodeKind> {
        CheckedNodeKind::all()
            .into_iter()
            .filter(|kind| admits(*kind))
            .collect()
    }

    /// A `creates` or `deletes` entry names exactly a `model`/`object_type`
    /// or `model`/`process` node, over every kind the closed vocabularies
    /// produce.
    ///
    /// Tracing: TC-053, FR-038-AC-12
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
    /// and a field entry exactly a `model`/`object_type` or
    /// `model`/`record_value_type` node; no systems form is a declaring node.
    ///
    /// Tracing: TC-056, FR-040-AC-2
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
            ]
        );
    }

    /// Tracing: TC-056, FR-040-AC-1
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
