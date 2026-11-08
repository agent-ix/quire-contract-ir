//! FR-322 operand-index checks before semantic-graph admission.
//!
//! The package document the shared reader built is walked on a heap stack, so
//! an index in any node reference has a source pointer (found only when a
//! refusal asks for it) without a second document tree and without native
//! recursion at deep sibling terms. Every table, worklist and key of the
//! stage is charged to the read's budget before it is made
//! (FR-038 "Budget-charged strict read", stage `NodeIndex`).

use super::native_join_budget::{NativeJoinBudget, PackageReadStage};
use super::read_charge::{self, Site};
use super::TermNode;
use super::{CheckedNodePathSegment, RawCheckedPackageWireV2, ValidationFailure};
use crate::checked_package::common::Step;
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::CheckedOperandEntry;
use crate::checked_package::shared::{
    push_index, push_key, CheckedPackageRefusal, CheckedPackageRefusalCode,
    CheckedPackageRefusalLocation, JsonPointer,
};
use quire_canonical::{Document, Items, Members, Node, NodeRef};
use std::sync::Arc;

/// The sentinel of a pointer-table slot whose key has no entry yet.
const NO_ENTRY: usize = usize::MAX;

struct OrderEntry<'k> {
    key: &'k super::CheckedNodeId,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
    size: usize,
    priority: u64,
}

/// The structural order of the resolved keys of one or more graphs, in
/// `CheckedNodeId::cmp` order, built once and charged once.
///
/// Equal keys, whichever document built them, share one entry, so an entry is
/// an equality token; the entries' positions in the order are dense ranks. A
/// key is found by its `Arc`'s address in a sorted pointer table and an
/// arbitrary resolved ID by descending the order, so no lookup allocates. The
/// order is a balanced tree whose entries enter children before parents, so a
/// parent's comparison reads the positions of its operands and never walks a
/// shared operand spine, whatever the depth.
///
/// Every table is charged to the budget's read storage before it is made: the
/// pointer table (`16` bytes a key), the entries, the worklist and the rank
/// table. Implements: FR-038 "Budget-charged strict read", stage `NodeIndex`.
pub(in crate::checked_package) struct StructuralOrder<'k> {
    entries: Vec<OrderEntry<'k>>,
    /// Key addresses in ascending order, with the entry of each.
    pointers: Vec<(*const super::CheckedNodeId, usize)>,
    root: Option<usize>,
    random: u64,
    /// The position of each entry in the order.
    positions: Vec<usize>,
}

impl<'k> StructuralOrder<'k> {
    /// Orders every key of `lists`, and every operand reachable from them,
    /// charging at the node-index stage's table index 0. `None` when an
    /// operand is not itself one of the listed keys.
    pub(super) fn build(
        lists: &[&'k [Arc<super::CheckedNodeId>]],
        budget: &mut NativeJoinBudget,
        site: Site,
    ) -> Result<Option<Self>, ValidationFailure> {
        let total = lists.iter().map(|list| list.len()).sum();
        let mut pointers: Vec<(*const super::CheckedNodeId, usize)> =
            read_charge::with_capacity(budget, site, total)?;
        for list in lists {
            pointers.extend(list.iter().map(|key| (Arc::as_ptr(key), NO_ENTRY)));
        }
        pointers.sort_unstable_by_key(|(pointer, _)| *pointer);
        pointers.dedup_by_key(|(pointer, _)| *pointer);
        let entries = read_charge::with_capacity(budget, site, pointers.len())?;
        let mut order = Self {
            entries,
            pointers,
            root: None,
            random: 0x9e37_79b9_7f4a_7c15,
            positions: Vec::new(),
        };
        let mut pending: Vec<(&'k super::CheckedNodeId, bool)> = Vec::new();
        for root in lists.iter().flat_map(|list| list.iter()) {
            read_charge::push(budget, site, &mut pending, (root.as_ref(), false))?;
            while let Some((key, finish)) = pending.pop() {
                let Some(slot) = order.slot(std::ptr::from_ref(key)) else {
                    return Ok(None);
                };
                if order.pointers[slot].1 != NO_ENTRY {
                    continue;
                }
                if finish {
                    let Some(entry) = order.insert(key) else {
                        return Ok(None);
                    };
                    order.pointers[slot].1 = entry;
                } else {
                    read_charge::push(budget, site, &mut pending, (key, true))?;
                    for segment in key.path.iter().rev() {
                        if let CheckedNodePathSegment::Node(child) = segment {
                            read_charge::push(budget, site, &mut pending, (child.as_ref(), false))?;
                        }
                    }
                }
            }
        }
        order.positions = read_charge::filled(budget, site, order.entries.len(), 0usize)?;
        let mut stack: Vec<usize> = Vec::new();
        let (mut at, mut next) = (order.root, 0usize);
        while at.is_some() || !stack.is_empty() {
            while let Some(current) = at {
                read_charge::push(budget, site, &mut stack, current)?;
                at = order.entries[current].left;
            }
            let Some(current) = stack.pop() else { break };
            order.positions[current] = next;
            next += 1;
            at = order.entries[current].right;
        }
        Ok(Some(order))
    }

    /// [`Self::build`] under a budget that charges nothing, for readers that
    /// hold no caller budget.
    pub(in crate::checked_package) fn build_unmetered(
        lists: &[&'k [Arc<super::CheckedNodeId>]],
    ) -> Option<Self> {
        let mut budget = NativeJoinBudget::unlimited();
        let site = Site::new(PackageReadStage::NodeIndex, 0, "node_index");
        Self::build(lists, &mut budget, site).ok().flatten()
    }

    /// The equality token of a key built into the order: keys that are equal
    /// share one, in whichever document they were built.
    pub(super) fn token(&self, key: &Arc<super::CheckedNodeId>) -> Option<usize> {
        let slot = self.slot(Arc::as_ptr(key))?;
        Some(self.pointers[slot].1).filter(|entry| *entry != NO_ENTRY)
    }

    /// The dense rank of a key built into the order, in `CheckedNodeId::cmp`
    /// order: equal keys share a rank.
    pub(in crate::checked_package) fn rank(
        &self,
        key: &Arc<super::CheckedNodeId>,
    ) -> Option<usize> {
        self.positions.get(self.token(key)?).copied()
    }

    #[allow(dead_code)] // Consumed by the graph validators' import index.
    /// The token of a resolved ID equal to a key of the order, found without
    /// allocating, or `None` when no key equals it or an operand of it is not
    /// a key of the order.
    pub(super) fn find(&self, id: &super::CheckedNodeId) -> Option<usize> {
        let mut at = self.root;
        while let Some(current) = at {
            at = match self.compare(id, self.entries[current].key)? {
                std::cmp::Ordering::Equal => return Some(current),
                std::cmp::Ordering::Less => self.entries[current].left,
                std::cmp::Ordering::Greater => self.entries[current].right,
            };
        }
        None
    }

    fn slot(&self, address: *const super::CheckedNodeId) -> Option<usize> {
        self.pointers
            .binary_search_by_key(&address, |(pointer, _)| *pointer)
            .ok()
    }

    fn entry_of(&self, operand: &Arc<super::CheckedNodeId>) -> Option<usize> {
        let slot = self.slot(Arc::as_ptr(operand))?;
        Some(self.pointers[slot].1).filter(|entry| *entry != NO_ENTRY)
    }

    fn size(&self, index: Option<usize>) -> usize {
        index.map_or(0, |at| self.entries[at].size)
    }

    fn position(&self, mut at: usize) -> usize {
        let mut position = self.size(self.entries[at].left);
        while let Some(parent) = self.entries[at].parent {
            if self.entries[parent].right == Some(at) {
                position += self.size(self.entries[parent].left) + 1;
            }
            at = parent;
        }
        position
    }

    /// `CheckedNodeId::cmp`, reading operand order from the tree. `None` when
    /// an operand of either ID is not in the order yet.
    fn compare(
        &self,
        left: &super::CheckedNodeId,
        right: &super::CheckedNodeId,
    ) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        let package = left.package.cmp(&right.package);
        if package != Ordering::Equal {
            return Some(package);
        }
        for (a, b) in left.path.iter().zip(&right.path) {
            let order = match (a, b) {
                (CheckedNodePathSegment::Name(a), CheckedNodePathSegment::Name(b)) => a.cmp(b),
                (CheckedNodePathSegment::Index(a), CheckedNodePathSegment::Index(b))
                | (CheckedNodePathSegment::Operand(a), CheckedNodePathSegment::Operand(b)) => {
                    a.cmp(b)
                }
                (CheckedNodePathSegment::Node(a), CheckedNodePathSegment::Node(b)) => {
                    let (a, b) = (self.entry_of(a)?, self.entry_of(b)?);
                    self.position(a).cmp(&self.position(b))
                }
                (CheckedNodePathSegment::Name(_), _) => Ordering::Less,
                (_, CheckedNodePathSegment::Name(_)) => Ordering::Greater,
                (CheckedNodePathSegment::Index(_), _) => Ordering::Less,
                (CheckedNodePathSegment::Operand(_), CheckedNodePathSegment::Index(_)) => {
                    Ordering::Greater
                }
                (CheckedNodePathSegment::Operand(_), _) => Ordering::Less,
                (
                    CheckedNodePathSegment::Dependency {
                        package: a,
                        node: x,
                    },
                    CheckedNodePathSegment::Dependency {
                        package: b,
                        node: y,
                    },
                ) => a.cmp(b).then(x.cmp(y)),
                (CheckedNodePathSegment::Dependency { .. }, CheckedNodePathSegment::Node(_)) => {
                    Ordering::Less
                }
                (CheckedNodePathSegment::Dependency { .. }, _) => Ordering::Greater,
                (CheckedNodePathSegment::Node(_), _) => Ordering::Greater,
            };
            if order != Ordering::Equal {
                return Some(order);
            }
        }
        Some(
            left.path
                .len()
                .cmp(&right.path.len())
                .then_with(|| left.ordinal.cmp(&right.ordinal)),
        )
    }

    fn rotate_up(&mut self, child: usize) {
        let Some(parent) = self.entries[child].parent else {
            return;
        };
        let grand = self.entries[parent].parent;
        if self.entries[parent].left == Some(child) {
            let moved = self.entries[child].right;
            self.entries[parent].left = moved;
            if let Some(moved) = moved {
                self.entries[moved].parent = Some(parent);
            }
            self.entries[child].right = Some(parent);
        } else {
            let moved = self.entries[child].left;
            self.entries[parent].right = moved;
            if let Some(moved) = moved {
                self.entries[moved].parent = Some(parent);
            }
            self.entries[child].left = Some(parent);
        }
        self.entries[parent].parent = Some(child);
        self.entries[child].parent = grand;
        if let Some(grand) = grand {
            if self.entries[grand].left == Some(parent) {
                self.entries[grand].left = Some(child);
            } else {
                self.entries[grand].right = Some(child);
            }
        } else {
            self.root = Some(child);
        }
        self.entries[parent].size =
            1 + self.size(self.entries[parent].left) + self.size(self.entries[parent].right);
        self.entries[child].size =
            1 + self.size(self.entries[child].left) + self.size(self.entries[child].right);
    }

    /// Enters `key`, or returns the entry of the key equal to it. `None` when
    /// an operand has no entry yet.
    fn insert(&mut self, key: &'k super::CheckedNodeId) -> Option<usize> {
        use std::cmp::Ordering;
        let mut parent = None;
        let mut at = self.root;
        let mut side = Ordering::Equal;
        while let Some(current) = at {
            side = self.compare(key, self.entries[current].key)?;
            if side == Ordering::Equal {
                return Some(current);
            }
            parent = Some(current);
            at = if side == Ordering::Less {
                self.entries[current].left
            } else {
                self.entries[current].right
            };
        }
        // The priority is for tree balance only; it never identifies a
        // package, file or tool, and it does not affect structural order.
        self.random = self
            .random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let index = self.entries.len();
        self.entries.push(OrderEntry {
            key,
            parent,
            left: None,
            right: None,
            size: 1,
            priority: self.random,
        });
        if let Some(parent) = parent {
            if side == Ordering::Less {
                self.entries[parent].left = Some(index);
            } else {
                self.entries[parent].right = Some(index);
            }
            let mut ancestor = Some(parent);
            while let Some(current) = ancestor {
                self.entries[current].size += 1;
                ancestor = self.entries[current].parent;
            }
        } else {
            self.root = Some(index);
        }
        while let Some(parent) = self.entries[index].parent {
            if self.entries[index].priority <= self.entries[parent].priority {
                break;
            }
            self.rotate_up(index);
        }
        Some(index)
    }
}

/// The charge site of the node-index stage at graph node `index`.
fn site(index: usize) -> Site {
    Site::new(PackageReadStage::NodeIndex, index, "node_index")
}

/// Walk the target's path without owning it. A document numbers children
/// before their container, so the first child not below the target holds it.
/// The same traversal counts the pointer first and writes it only after payment.
fn locate<'d>(document: &'d Document, target: NodeRef<'d>, mut step: impl FnMut(Step<'d>)) {
    let mut value = document.root();
    let target = target.id();
    while value.id() != target {
        match value.node() {
            Node::Object(mut members) => {
                let Some((name, child)) = members.find(|(_, child)| child.id() >= target) else {
                    break;
                };
                step(Step::Key(name));
                value = child;
            }
            Node::Array(items) => {
                let Some((position, child)) =
                    items.enumerate().find(|(_, child)| child.id() >= target)
                else {
                    break;
                };
                step(Step::Index(position));
                value = child;
            }
            Node::Null | Node::Bool(_) | Node::Number(_) | Node::String(_) => break,
        }
    }
}

fn step_bytes(step: Step<'_>) -> usize {
    match step {
        Step::Key(key) => key.bytes().fold(1usize, |bytes, byte| {
            bytes.saturating_add(if matches!(byte, b'~' | b'/') { 2 } else { 1 })
        }),
        Step::Index(mut index) => {
            let mut bytes = 2;
            while index >= 10 {
                index /= 10;
                bytes += 1;
            }
            bytes
        }
    }
}

/// Pay for the complete diagnostic before requesting its pointer, location
/// vector or boxed record. The graph holder comes from the document path,
/// including the row holder of `semantic_graph.operand_rows`.
fn malformed_at(
    document: &Document,
    target: NodeRef<'_>,
    budget: &mut NativeJoinBudget,
) -> ValidationFailure {
    use super::read_charge::ReadStorage;
    let mut length = 0usize;
    let mut depth = 0usize;
    let mut graph = false;
    let mut node_list = false;
    let mut index = 0;
    locate(document, target, |step| {
        length = length.saturating_add(step_bytes(step));
        match (depth, step) {
            (0, Step::Key("semantic_graph")) => graph = true,
            (1, Step::Key("nodes" | "operand_rows")) if graph => node_list = true,
            (2, Step::Index(holder)) if node_list => index = holder,
            _ => {}
        }
        depth += 1;
    });
    let bytes = (std::mem::size_of::<CheckedPackageRefusal>() as u128)
        .saturating_add(std::mem::size_of::<CheckedPackageRefusalLocation>() as u128)
        .saturating_add(length as u128);
    if budget
        .reserve_diagnostic(
            bytes,
            site(index),
            super::CheckedPackagePendingOutcome::Refused(CheckedPackageRefusalCode::MalformedWire),
        )
        .is_err()
    {
        return budget.denial();
    }
    let mut pointer = String::with_capacity(length);
    locate(document, target, |step| match step {
        Step::Key(key) => push_key(&mut pointer, key),
        Step::Index(index) => push_index(&mut pointer, index),
    });
    debug_assert_eq!(pointer.len(), length);
    ValidationFailure::refused(
        CheckedPackageRefusalCode::MalformedWire,
        JsonPointer::from_escaped(pointer),
    )
}

/// A closed node-index position is already a borrowed path; use the same
/// caller budget and diagnostic policy as the document locator.
fn malformed_path(
    budget: &mut NativeJoinBudget,
    index: usize,
    path: super::stage_storage::Path<'_>,
) -> ValidationFailure {
    super::stage_storage::refuse(
        &mut super::WorkMeter::over(budget),
        &super::stage_storage::Site::new(PackageReadStage::NodeIndex, index, "node_index", path),
        super::stage_storage::Refusal::new(CheckedPackageRefusalCode::MalformedWire, path),
    )
}

/// One operand segment of the document, in document order by `id`.
struct SegmentSite<'d> {
    /// The segment's number, whose id orders the sites and names its pointer.
    id: NodeRef<'d>,
    value: u64,
    /// An operand slot (`{"operand": k}`) rather than an index segment.
    slot: bool,
    /// The node whose members hold this segment; `None` in a document-held id.
    holder: Option<usize>,
    /// Inside a foreign `{package, node}` key, which names a dependency's
    /// graph of this many nodes and takes no part in the placement check.
    foreign_graph: Option<usize>,
}

/// One entry of an operand row.
struct RowSite<'d> {
    id: NodeRef<'d>,
    entry: RowEntry<'d>,
}

#[derive(Clone, Copy)]
enum RowEntry<'d> {
    Node(u64),
    Dependency { package: &'d str, node: u64 },
}

/// Where the walk is in the package, to tell node-held ids from document-held.
#[derive(Clone, Copy)]
enum Scope {
    Root,
    Graph,
    Nodes,
    Node(usize),
    Rows,
    Row,
    Other,
}

#[derive(Clone, Copy)]
struct Walk {
    graph_nodes: usize,
    scope: Scope,
    foreign_graph: Option<usize>,
}

impl Walk {
    /// The graph node index the walk's charges are made at.
    fn index(self) -> usize {
        match self.scope {
            Scope::Node(holder) => holder,
            Scope::Root | Scope::Graph | Scope::Nodes | Scope::Rows | Scope::Row | Scope::Other => {
                0
            }
        }
    }
}

/// Detect a structural operand embedded in an encoded checked-package path.
/// The generic `CheckedNodeId` encoder still writes such values for FR-201
/// structural preimages; the V2 package envelope has the stricter index wire.
pub(super) fn contains_inline_node_path(bytes: &[u8]) -> bool {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Key {
        Other,
        Package,
        Path,
        Ordinal,
        Node,
        Operand,
        Term,
    }
    enum Frame {
        Object {
            start: usize,
            members: usize,
            keys: u8,
            current: Key,
            path_bad: bool,
            path_index: bool,
            term_dependency_reference: bool,
            node_index: bool,
        },
        Array {
            path: bool,
            bad: bool,
            index: bool,
        },
    }

    let mut stack = Vec::<Frame>::new();
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'{' => stack.push(Frame::Object {
                start: at,
                members: 0,
                keys: 0,
                current: Key::Other,
                path_bad: false,
                path_index: false,
                term_dependency_reference: false,
                node_index: false,
            }),
            b'[' => {
                if let Some(Frame::Array {
                    path: true, bad, ..
                }) = stack.last_mut()
                {
                    *bad = true;
                }
                let path = matches!(
                    stack.last(),
                    Some(Frame::Object {
                        current: Key::Path,
                        ..
                    })
                );
                stack.push(Frame::Array {
                    path,
                    bad: false,
                    index: false,
                });
            }
            b'"' => {
                let start = at + 1;
                at = start;
                while let Some(&part) = bytes.get(at) {
                    if part == b'\\' {
                        at += 2;
                    } else if part == b'"' {
                        break;
                    } else {
                        at += 1;
                    }
                }
                if bytes.get(at + 1) == Some(&b':') {
                    if let Some(Frame::Object {
                        members,
                        keys,
                        current,
                        ..
                    }) = stack.last_mut()
                    {
                        *members += 1;
                        *current = match bytes.get(start..at) {
                            Some(b"package") => Key::Package,
                            Some(b"path") => Key::Path,
                            Some(b"ordinal") => Key::Ordinal,
                            Some(b"node") => Key::Node,
                            Some(b"operand") => Key::Operand,
                            Some(b"term") => Key::Term,
                            _ => Key::Other,
                        };
                        *keys |= match current {
                            Key::Package => 1,
                            Key::Path => 2,
                            Key::Ordinal => 4,
                            Key::Node => 8,
                            Key::Operand => 16,
                            Key::Term => 0,
                            Key::Other => 0,
                        };
                    }
                } else if let Some(Frame::Object {
                    current: Key::Term,
                    term_dependency_reference,
                    ..
                }) = stack.last_mut()
                {
                    if bytes.get(start..at) == Some(b"dependency_reference") {
                        *term_dependency_reference = true;
                    }
                }
            }
            b'}' => {
                let Some(Frame::Object {
                    start,
                    members,
                    keys,
                    path_bad,
                    path_index,
                    term_dependency_reference,
                    node_index,
                    ..
                }) = stack.pop()
                else {
                    return true;
                };
                if members == 3 && keys == 7 && path_bad {
                    return true;
                }
                if term_dependency_reference && node_index {
                    return true;
                }
                let is_node_id = members == 3 && keys == 7;
                if let Some(Frame::Object {
                    current: Key::Node,
                    node_index,
                    ..
                }) = stack.last_mut()
                {
                    if is_node_id {
                        *node_index = path_index;
                    }
                }
                if let Some(Frame::Array {
                    path: true,
                    bad,
                    index,
                }) = stack.last_mut()
                {
                    let prefix: &[u8] = if keys == 16 {
                        b"{\"operand\":"
                    } else {
                        b"{\"node\":"
                    };
                    let is_index = members == 1
                        && (keys == 8 || keys == 16)
                        && bytes
                            .get(start + prefix.len()..at)
                            .and_then(|raw| std::str::from_utf8(raw).ok())
                            .and_then(|raw| raw.parse::<u64>().ok())
                            .is_some_and(|index| index <= 9_007_199_254_740_991);
                    *bad |= !is_index;
                    *index |= is_index;
                }
            }
            b']' => {
                let Some(Frame::Array { bad, index, .. }) = stack.pop() else {
                    return true;
                };
                if let Some(Frame::Object {
                    current: Key::Path,
                    path_bad,
                    path_index,
                    ..
                }) = stack.last_mut()
                {
                    *path_bad |= bad;
                    *path_index |= index;
                }
            }
            b't' | b'f' | b'n' | b'-' | b'0'..=b'9' => {
                if let Some(Frame::Array {
                    path: true, bad, ..
                }) = stack.last_mut()
                {
                    *bad = true;
                }
            }
            _ => {}
        }
        at += 1;
    }
    !stack.is_empty()
}

/// `resolve_id` with the copy's blocks (path, package, names) charged first.
pub(super) fn resolve_id_charged(
    wire: &super::CheckedNodeId,
    keys: &[Arc<super::CheckedNodeId>],
    meter: &mut super::WorkMeter,
) -> Result<Option<super::CheckedNodeId>, ValidationFailure> {
    let mut bytes = wire
        .path
        .len()
        .saturating_mul(std::mem::size_of::<CheckedNodePathSegment>())
        .saturating_add(wire.package.len());
    for segment in &wire.path {
        if let CheckedNodePathSegment::Name(name) | CheckedNodePathSegment::Dependency { package: name, .. } = segment {
            bytes = bytes.saturating_add(name.len());
        }
    }
    meter.reserve(bytes)?;
    Ok(resolve_id(wire, keys))
}

/// Rebuild an ID written in one package document against that document's
/// graph. The returned operands share the already rebuilt graph keys. The
/// ID must be in the reader's resolved form, whose operands are index segments.
pub(super) fn resolve_id(
    wire: &super::CheckedNodeId,
    keys: &[Arc<super::CheckedNodeId>],
) -> Option<super::CheckedNodeId> {
    let mut path = Vec::with_capacity(wire.path.len());
    for segment in &wire.path {
        path.push(match segment {
            CheckedNodePathSegment::Name(name) => CheckedNodePathSegment::Name(name.clone()),
            CheckedNodePathSegment::Index(index) => {
                CheckedNodePathSegment::Node(Arc::clone(keys.get(usize::try_from(*index).ok()?)?))
            }
            // A dependency position names a node of another graph; it stays
            // as written, the resolved form of a foreign operand.
            CheckedNodePathSegment::Dependency { .. } => segment.clone(),
            CheckedNodePathSegment::Node(_) | CheckedNodePathSegment::Operand(_) => return None,
        });
    }
    Some(super::CheckedNodeId {
        package: wire.package.clone(),
        path,
        ordinal: wire.ordinal,
    })
}

/// Compare two body values from distinct package documents, resolving every
/// node ID against the graph of the document that contains it.
pub(super) fn equivalent_values(
    left: super::TermNode<'_>,
    right: super::TermNode<'_>,
    left_keys: &[Arc<super::CheckedNodeId>],
    right_keys: &[Arc<super::CheckedNodeId>],
    left_row: &[CheckedOperandEntry],
    right_row: &[CheckedOperandEntry],
    meter: &mut super::WorkMeter,
) -> Result<bool, ValidationFailure> {
    use quire_canonical::Node;
    let site = super::read_charge::Site::new(super::PackageReadStage::Graph, 0, "import_body");
    let mut pending = meter.vec(1)?;
    pending.push((left, right));
    while let Some((left, right)) = pending.pop() {
        if left.is_object() && right.is_object() {
            // FR-375 external keys are the exception to document-local
            // index scope: their `node` is scoped by the outer package.
            // Both documents select that dependency under the same key,
            // so compare its wire ID directly instead of resolving it in
            // either containing document's graph.
            if left.exact_members(&["package", "node"]) && right.exact_members(&["package", "node"])
            {
                if left.get("package").and_then(super::TermNode::as_str)
                    != right.get("package").and_then(super::TermNode::as_str)
                {
                    return Ok(false);
                }
                let (Some(a), Some(b)) = (left.get("node"), right.get("node")) else {
                    return Ok(false);
                };
                super::read_charge::push_charged(meter, site, &mut pending, (a, b))?;
                continue;
            }
            let a_shape = left.exact_members(&["package", "path", "ordinal"]);
            let b_shape = right.exact_members(&["package", "path", "ordinal"]);
            if a_shape || b_shape {
                let a = left
                    .node_id_in(&mut *meter, site, left_row)
                    .map_err(|_| super::read_charge::ReadStorage::denial(meter))?
                    .valid();
                let b = right
                    .node_id_in(&mut *meter, site, right_row)
                    .map_err(|_| super::read_charge::ReadStorage::denial(meter))?
                    .valid();
                let (Some(a), Some(b)) = (a, b) else {
                    return Ok(false);
                };
                let same = match (
                    resolve_id_charged(&a, left_keys, meter)?,
                    resolve_id_charged(&b, right_keys, meter)?,
                ) {
                    (Some(a), Some(b)) => meter.compare_ids(&a, &b)?.is_eq(),
                    (None, None) => true,
                    _ => false,
                };
                if !same {
                    return Ok(false);
                }
            } else {
                let (Some(a), Some(b)) = (left.members(), right.members()) else {
                    return Ok(false);
                };
                if a.len() != b.len() {
                    return Ok(false);
                }
                for (name, value) in a {
                    let Some(other) = right.get(name) else {
                        return Ok(false);
                    };
                    super::read_charge::push_charged(meter, site, &mut pending, (value, other))?;
                }
            }
        } else if left.is_array() && right.is_array() {
            let (Some(a), Some(b)) = (left.items(), right.items()) else {
                return Ok(false);
            };
            if a.len() != b.len() {
                return Ok(false);
            }
            for pair in a.zip(b) {
                super::read_charge::push_charged(meter, site, &mut pending, pair)?;
            }
        } else {
            let same = match (left.node_ref().node(), right.node_ref().node()) {
                (Node::Null, Node::Null) => true,
                (Node::Bool(a), Node::Bool(b)) => a == b,
                (Node::String(a), Node::String(b)) => a == b,
                (Node::Number(a), Node::Number(b)) => a.text() == b.text(),
                _ => false,
            };
            if !same {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Resolve graph-local node operands of the FR-035 ContractPackage projection.
/// The caller owns refusal coding.
#[ix_trace_rs::string_edge]
pub(super) fn rebuild_graph_keys(
    nodes: &[super::RawSemanticNodeV2],
    refuse: impl Fn(JsonPointer) -> ValidationFailure,
    budget: &mut NativeJoinBudget,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    rebuild_keys(
        nodes.len(),
        |position| &nodes[position].node_id,
        |position, slot| {
            JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position)
                .key("node_id")
                .key("path")
                .index(slot)
                .key("node")
        },
        refuse,
        budget,
    )
}

/// Resolve one document-local table of `count` IDs, each holding index
/// segments, without recursive key construction. The caller supplies the
/// table-specific pointer for an operand.
///
/// Every table, worklist and key is charged to the budget's read storage
/// before it is made: the edge tables and the reverse table at node 0, and
/// each node's key (its package, names, path and `Arc` block) at that node.
pub(super) fn rebuild_keys<'a>(
    count: usize,
    id_of: impl Fn(usize) -> &'a super::CheckedNodeId,
    pointer: impl Fn(usize, usize) -> JsonPointer,
    refuse: impl Fn(JsonPointer) -> ValidationFailure,
    budget: &mut NativeJoinBudget,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    rebuild_keys_with_failure(
        count,
        id_of,
        |position, slot, _budget| refuse(pointer(position, slot)),
        budget,
    )
}

/// One key-building implementation; the strict document caller supplies a
/// borrowed, caller-paid fault and other callers retain their own policy.
fn rebuild_keys_with_failure<'a>(
    count: usize,
    id_of: impl Fn(usize) -> &'a super::CheckedNodeId,
    mut refuse: impl FnMut(usize, usize, &mut NativeJoinBudget) -> ValidationFailure,
    budget: &mut NativeJoinBudget,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    // The graph position a segment names, or `None` for a segment that is
    // not an operand reference of this table.
    let target_of = |segment: &CheckedNodePathSegment| -> Option<Option<usize>> {
        match segment {
            CheckedNodePathSegment::Name(_) | CheckedNodePathSegment::Node(_) => Some(None),
            CheckedNodePathSegment::Index(index) => Some(Some(usize::try_from(*index).ok()?)),
            CheckedNodePathSegment::Operand(_) | CheckedNodePathSegment::Dependency { .. } => None,
        }
    };
    // The edges as one table: the first pass finds the first bad operand in
    // node and slot order and counts the edges; the second fills the rows.
    let mut edge_count = 0usize;
    for position in 0..count {
        for (slot, segment) in id_of(position).path.iter().enumerate() {
            match target_of(segment) {
                Some(None) => {}
                Some(Some(target)) if target < count => edge_count += 1,
                Some(Some(_)) | None => return Err(refuse(position, slot, budget)),
            }
        }
    }
    let tables = site(0);
    let mut starts: Vec<usize> = read_charge::with_capacity(budget, tables, count + 1)?;
    let mut edges: Vec<(usize, usize)> = read_charge::with_capacity(budget, tables, edge_count)?;
    for position in 0..count {
        starts.push(edges.len());
        for (slot, segment) in id_of(position).path.iter().enumerate() {
            if let Some(Some(target)) = target_of(segment) {
                edges.push((slot, target));
            }
        }
    }
    starts.push(edges.len());
    let outgoing = |node: usize| -> &[(usize, usize)] {
        match (starts.get(node), starts.get(node + 1)) {
            (Some(&from), Some(&to)) => edges.get(from..to).unwrap_or_default(),
            _ => &[],
        }
    };
    // Kosaraju's two heap-backed passes make the cycle decision linear in
    // nodes plus operand segments, independent of graph order and chain depth.
    let mut seen = read_charge::filled(budget, tables, count, false)?;
    let mut order: Vec<usize> = read_charge::with_capacity(budget, tables, count)?;
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for start in 0..count {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        read_charge::push(budget, tables, &mut stack, (start, 0))?;
        while let Some((node, cursor)) = stack.pop() {
            if let Some((_, target)) = outgoing(node).get(cursor) {
                // The node's frame is replaced in place: a pop then a push
                // never grows the list.
                stack.push((node, cursor + 1));
                if !seen[*target] {
                    seen[*target] = true;
                    read_charge::push(budget, tables, &mut stack, (*target, 0))?;
                }
            } else {
                order.push(node);
            }
        }
    }
    // The reverse table, by counting sort over the edge targets.
    let mut reverse_starts = read_charge::filled(budget, tables, count + 1, 0usize)?;
    for &(_, target) in &edges {
        reverse_starts[target] += 1;
    }
    for node in 1..count {
        reverse_starts[node] += reverse_starts[node - 1];
    }
    reverse_starts[count] = edges.len();
    let mut reverse = read_charge::filled(budget, tables, edges.len(), 0usize)?;
    for node in (0..count).rev() {
        for &(_, target) in outgoing(node) {
            reverse_starts[target] -= 1;
            reverse[reverse_starts[target]] = node;
        }
    }
    let mut component = read_charge::filled(budget, tables, count, usize::MAX)?;
    for &start in order.iter().rev() {
        if component[start] != usize::MAX {
            continue;
        }
        component[start] = start;
        stack.clear();
        read_charge::push(budget, tables, &mut stack, (start, 0))?;
        while let Some((node, _)) = stack.pop() {
            let from = reverse_starts[node];
            let to = reverse_starts[node + 1];
            for &previous in &reverse[from..to] {
                if component[previous] == usize::MAX {
                    component[previous] = start;
                    read_charge::push(budget, tables, &mut stack, (previous, 0))?;
                }
            }
        }
    }
    for node in 0..count {
        for &(slot, target) in outgoing(node) {
            if component[node] == component[target] {
                return Err(refuse(node, slot, budget));
            }
        }
    }
    let mut keys: Vec<Option<Arc<super::CheckedNodeId>>> =
        read_charge::filled(budget, tables, count, None)?;
    for node in order {
        let source = id_of(node);
        let at = site(node);
        let mut path = read_charge::with_capacity(budget, at, source.path.len())?;
        for segment in &source.path {
            path.push(match segment {
                CheckedNodePathSegment::Name(name) => {
                    CheckedNodePathSegment::Name(read_charge::boxed_str(budget, at, name)?)
                }
                CheckedNodePathSegment::Node(key) => CheckedNodePathSegment::Node(Arc::clone(key)),
                CheckedNodePathSegment::Index(_)
                | CheckedNodePathSegment::Operand(_)
                | CheckedNodePathSegment::Dependency { .. } => {
                    let target = target_of(segment)
                        .flatten()
                        .expect("operand was checked in bounds");
                    CheckedNodePathSegment::Node(Arc::clone(
                        keys[target].as_ref().expect("target key built first"),
                    ))
                }
            });
        }
        let package = read_charge::boxed_str(budget, at, &source.package)?;
        keys[node] = Some(read_charge::new_arc(
            budget,
            at,
            super::CheckedNodeId {
                package,
                path,
                ordinal: source.ordinal,
            },
        )?);
    }
    let mut output = read_charge::with_capacity(budget, tables, count)?;
    for key in keys {
        output.push(key.expect("each graph node key built"));
    }
    Ok(output)
}

/// The cursor over a container's children.
enum Children<'d> {
    /// An object's members; `external` is the node count of the dependency
    /// graph the value of `node` is scoped by, when this is a foreign key.
    Members(Members<'d>, Option<usize>),
    /// An array's items and the position of the next.
    Items(Items<'d>, usize),
}

/// A container being walked and its position in the package.
struct Frame<'d> {
    children: Children<'d>,
    walk: Walk,
}

/// The number a value holds as a `u64`, when it is one.
fn as_u64(value: NodeRef<'_>) -> Option<u64> {
    match value.node() {
        Node::Number(number) => number.text().parse().ok(),
        Node::Null | Node::Bool(_) | Node::String(_) | Node::Array(_) | Node::Object(_) => None,
    }
}

/// The walk of [`collect_sites`].
struct Collector<'d, 'e, 'b> {
    document: &'d Document,
    evidence: &'e CheckedPackageEvidence,
    budget: &'b mut NativeJoinBudget,
    sites: Vec<SegmentSite<'d>>,
    row_sites: Vec<RowSite<'d>>,
}

impl<'d> Collector<'d, '_, '_> {
    /// Enters one value: records the operand segments an ID object holds and
    /// returns the frame of a container, which the caller walks.
    fn enter(
        &mut self,
        value: NodeRef<'d>,
        walk: Walk,
    ) -> Result<Option<Frame<'d>>, ValidationFailure> {
        match value.node() {
            Node::Object(members) => {
                self.check_dependency_reference(&members)?;
                self.record_segments(&members, walk)?;
                // FR-375's external key is scoped by its outer package,
                // regardless of the package named inside its node ID.
                let is_external_key = members.len() == 2
                    && members.clone().any(|(name, _)| name == "node")
                    && members.clone().any(|(name, _)| name == "package");
                let external = is_external_key.then(|| {
                    members
                        .clone()
                        .find(|(name, _)| *name == "package")
                        .and_then(|(_, package)| match package.node() {
                            Node::String(package) => Some(package),
                            _ => None,
                        })
                        .and_then(|package| self.evidence.dependency_package(package))
                        .map_or(usize::MAX, |package| package.resolved_graph().nodes.len())
                });
                Ok(Some(Frame {
                    children: Children::Members(members, external),
                    walk,
                }))
            }
            Node::Array(items) => Ok(Some(Frame {
                children: Children::Items(items, 0),
                walk,
            })),
            Node::Null | Node::Bool(_) | Node::Number(_) | Node::String(_) => Ok(None),
        }
    }

    /// A `dependency_reference` term names its target with a document-held
    /// ID whose path holds no operand object.
    fn check_dependency_reference(
        &mut self,
        members: &Members<'d>,
    ) -> Result<(), ValidationFailure> {
        let is_reference = members.clone().any(|(name, value)| {
            name == "term" && matches!(value.node(), Node::String("dependency_reference"))
        });
        if !is_reference {
            return Ok(());
        }
        let Some((_, node)) = members.clone().find(|(name, _)| *name == "node") else {
            return Ok(());
        };
        let Node::Object(node_members) = node.node() else {
            return Err(malformed_at(self.document, node, self.budget));
        };
        let Some((_, path)) = node_members.clone().find(|(name, _)| *name == "path") else {
            return Ok(());
        };
        let Node::Array(segments) = path.node() else {
            return Err(malformed_at(self.document, path, self.budget));
        };
        for segment in segments {
            if matches!(segment.node(), Node::Object(_)) {
                return Err(malformed_at(self.document, segment, self.budget));
            }
        }
        Ok(())
    }

    /// Records the `{"operand": k}` and `{"node": i}` segments of an ID object.
    fn record_segments(
        &mut self,
        members: &Members<'d>,
        walk: Walk,
    ) -> Result<(), ValidationFailure> {
        if members.len() != 3
            || !["package", "path", "ordinal"]
                .iter()
                .all(|wanted| members.clone().any(|(name, _)| name == *wanted))
        {
            return Ok(());
        }
        let Some((_, path)) = members.clone().find(|(name, _)| *name == "path") else {
            return Ok(());
        };
        let Node::Array(segments) = path.node() else {
            return Err(malformed_at(self.document, path, self.budget));
        };
        let holder = match walk.scope {
            Scope::Node(holder) => Some(holder),
            _ => None,
        };
        for segment in segments {
            let Node::Object(mut object) = segment.node() else {
                continue;
            };
            if object.len() == 2
                && object.clone().any(|(key, _)| key == "package")
                && object.clone().any(|(key, _)| key == "node")
            {
                return Err(malformed_at(self.document, segment, self.budget));
            }
            if object.len() != 1 {
                continue;
            }
            let Some((name, member)) = object.next() else {
                continue;
            };
            if !matches!(name, "node" | "operand") {
                continue;
            }
            let Some(value) = as_u64(member) else {
                return Err(malformed_at(self.document, segment, self.budget));
            };
            read_charge::push(
                self.budget,
                site(walk.index()),
                &mut self.sites,
                SegmentSite {
                    id: member,
                    value,
                    slot: name == "operand",
                    holder,
                    foreign_graph: walk.foreign_graph,
                },
            )?;
        }
        Ok(())
    }
}

/// Collect the operand segments and operand row entries of the package
/// `document` in document order. A segment object is `{"operand": k}` or
/// `{"node": i}` as the sole member of a path item of a node ID.
fn collect_sites<'d>(
    document: &'d Document,
    node_count: usize,
    evidence: &CheckedPackageEvidence,
    budget: &mut NativeJoinBudget,
) -> Result<(Vec<SegmentSite<'d>>, Vec<RowSite<'d>>), ValidationFailure> {
    let mut collector = Collector {
        document,
        evidence,
        budget,
        sites: Vec::new(),
        row_sites: Vec::new(),
    };
    let root = Walk {
        graph_nodes: node_count,
        scope: Scope::Root,
        foreign_graph: None,
    };
    let mut stack: Vec<Frame<'_>> = Vec::new();
    if let Some(frame) = collector.enter(document.root(), root)? {
        read_charge::push(collector.budget, site(0), &mut stack, frame)?;
    }
    while let Some(frame) = stack.last_mut() {
        let walk = frame.walk;
        let in_items = matches!(frame.children, Children::Items(..));
        let next = match &mut frame.children {
            Children::Members(members, external) => members.next().map(|(name, child)| {
                let mut child_walk = walk;
                child_walk.scope = match (walk.scope, name) {
                    (Scope::Root, "semantic_graph") => Scope::Graph,
                    (Scope::Graph, "nodes") => Scope::Nodes,
                    (Scope::Graph, "operand_rows") => Scope::Rows,
                    (Scope::Node(holder), _) => Scope::Node(holder),
                    _ => Scope::Other,
                };
                if let (Some(nodes), "node") = (*external, name) {
                    child_walk.graph_nodes = nodes;
                    child_walk.foreign_graph = Some(nodes);
                }
                (child, child_walk)
            }),
            Children::Items(items, position) => items.next().map(|child| {
                let mut child_walk = walk;
                child_walk.scope = match walk.scope {
                    Scope::Nodes => Scope::Node(*position),
                    Scope::Rows => Scope::Row,
                    Scope::Node(holder) => Scope::Node(holder),
                    _ => Scope::Other,
                };
                *position += 1;
                (child, child_walk)
            }),
        };
        let Some((child, child_walk)) = next else {
            stack.pop();
            continue;
        };
        if let (Scope::Row, true) = (walk.scope, in_items) {
            let entry = if let Some(value) = as_u64(child) {
                RowEntry::Node(value)
            } else {
                let value = super::TermNode::from_ref(child);
                if !value.exact_members(&["package", "node"]) {
                    return Err(malformed_at(document, child, collector.budget));
                }
                let (Some(package), Some(node)) = (
                    value.get("package").and_then(super::TermNode::as_str),
                    value.get("node").and_then(super::TermNode::as_u64),
                ) else {
                    return Err(malformed_at(document, child, collector.budget));
                };
                RowEntry::Dependency { package, node }
            };
            read_charge::push(
                collector.budget,
                site(0),
                &mut collector.row_sites,
                RowSite { id: child, entry },
            )?;
            continue;
        }
        if let Some(frame) = collector.enter(child, child_walk)? {
            read_charge::push(
                collector.budget,
                site(child_walk.index()),
                &mut stack,
                frame,
            )?;
        }
    }
    let Collector {
        mut sites,
        mut row_sites,
        ..
    } = collector;
    sites.sort_unstable_by_key(|site| site.id.id());
    row_sites.sort_unstable_by_key(|site| site.id.id());
    Ok((sites, row_sites))
}

/// Run the FR-322 operand checks in their order, before any graph meaning
/// check, then rebuild every node's structural key. The first defect of the
/// first step that has one is refused as `malformed_wire`.
///
/// `wire` is the resolved graph: each node-held operand slot already reads as
/// the index segment of the node its row names (see [`resolve_slots`]).
/// Every table and worklist is charged at the `NodeIndex` stage.
#[ix_trace_rs::string_edge]
pub(super) fn check(
    document: &Document,
    wire: &RawCheckedPackageWireV2,
    evidence: &CheckedPackageEvidence,
    budget: &mut NativeJoinBudget,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    let node_count = wire.semantic_graph.nodes.len();
    let rows = &wire.semantic_graph.operand_rows;
    let graph_at = super::stage_storage::Path::root().key("semantic_graph");
    // 1. Rows.
    if rows.len() != node_count {
        return Err(malformed_path(budget, 0, graph_at.key("operand_rows")));
    }
    let (sites, row_sites) = collect_sites(document, node_count, evidence, budget)?;
    // 2. Placement. A foreign key's segments name its dependency's graph.
    if let Some(site) = sites
        .iter()
        .find(|site| site.foreign_graph.is_none() && site.slot != site.holder.is_some())
    {
        return Err(malformed_at(document, site.id, budget));
    }
    // 3. Range.
    let out_of_range = sites.iter().find_map(|site| {
        let limit = match (site.slot, site.holder, site.foreign_graph) {
            (true, Some(holder), None) => rows[holder].len(),
            (false, _, Some(graph)) => graph,
            (false, _, None) => node_count,
            (true, _, _) => 0,
        };
        usize::try_from(site.value)
            .ok()
            .is_none_or(|value| value >= limit)
            .then_some(site.id)
    });
    // A row entry names a node of this graph, or of a supplied dependency's.
    let bad_entry = row_sites.iter().find_map(|site| {
        let (value, limit) = match site.entry {
            RowEntry::Node(value) => (value, node_count),
            RowEntry::Dependency { package, node } => (
                node,
                evidence
                    .dependency_package(package)
                    .map_or(0, |package| package.resolved_graph().nodes.len()),
            ),
        };
        usize::try_from(value)
            .ok()
            .is_none_or(|value| value >= limit)
            .then_some(site.id)
    });
    if let Some(at) = [out_of_range, bad_entry]
        .into_iter()
        .flatten()
        .min_by_key(|at| at.id())
    {
        return Err(malformed_at(document, at, budget));
    }
    // 4. Numbering by first occurrence, node by node in graph order. The
    // sites are in document order, so a node's slots are consecutive and in
    // ascending node order.
    let longest_row = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut entries: Vec<(&CheckedOperandEntry, usize)> =
        read_charge::with_capacity(budget, site(0), longest_row)?;
    let mut slots = sites
        .iter()
        .filter(|site| site.slot && site.holder.is_some() && site.foreign_graph.is_none())
        .peekable();
    for (node, row) in rows.iter().enumerate() {
        let mut named = 0u64;
        while let Some(site) = slots.next_if(|site| site.holder == Some(node)) {
            if site.value > named {
                return Err(malformed_at(document, site.id, budget));
            }
            named = named.max(site.value + 1);
        }
        // The first row position that is unnamed or repeats an earlier entry.
        entries.clear();
        entries.extend(row.iter().zip(0..));
        entries.sort_unstable();
        let repeated = entries
            .windows(2)
            .filter(|pair| pair[0].0 == pair[1].0)
            .map(|pair| pair[1].1)
            .min();
        let unnamed = usize::try_from(named)
            .ok()
            .filter(|&named| named < row.len());
        if let Some(position) = [unnamed, repeated].into_iter().flatten().min() {
            return Err(malformed_path(
                budget,
                node,
                graph_at.key("operand_rows").index(node).index(position),
            ));
        }
    }
    drop(entries);
    drop(sites);
    drop(row_sites);
    // 5. Cycle, then the keys themselves.
    let keys = rebuild_keys_with_failure(
        node_count,
        |position| &wire.semantic_graph.nodes[position].node_id,
        |position, slot, budget| {
            malformed_path(
                budget,
                position,
                graph_at
                    .key("nodes")
                    .index(position)
                    .key("node_id")
                    .key("path")
                    .index(slot)
                    .key("operand"),
            )
        },
        budget,
    )?;
    read_charge::end_stage(budget);
    Ok(keys)
}

/// Rewrite every node-held operand slot of the checked nodes to the document
/// index segment of the node its row names. The rest of the reader compares
/// operands by graph index, so a slot is read through its row once, here.
/// Every slot was range-checked by [`check`].
///
/// The worklist of each body is charged at the `owner_schema` form of the
/// wire checks, which this rewrite prepares; a rewritten segment reuses its
/// own key text and allocates nothing.
pub(super) fn resolve_slots(
    wire: &mut RawCheckedPackageWireV2,
    bodies: &[super::CheckedJson],
    budget: &mut NativeJoinBudget,
) -> Result<(), ValidationFailure> {
    let graph = &mut wire.semantic_graph;
    let rows = &graph.operand_rows;
    for (position, ((node, row), body)) in graph.nodes.iter_mut().zip(rows).zip(bodies).enumerate()
    {
        let row_site = Site::new(PackageReadStage::WireChecks, position, "owner_schema");
        let mut to_index = |id: &mut super::CheckedNodeId| -> Result<(), ValidationFailure> {
            for segment in &mut id.path {
                if let CheckedNodePathSegment::Operand(slot) = segment {
                    if let Some(entry) = usize::try_from(*slot).ok().and_then(|slot| row.get(slot))
                    {
                        *segment = match entry {
                            CheckedOperandEntry::Node(entry) => {
                                CheckedNodePathSegment::Index(*entry)
                            }
                            CheckedOperandEntry::Dependency(operand) => {
                                CheckedNodePathSegment::Dependency {
                                    package: read_charge::boxed_str(
                                        budget,
                                        row_site,
                                        &operand.package,
                                    )?,
                                    node: operand.node,
                                }
                            }
                        };
                    }
                }
            }
            Ok(())
        };
        to_index(&mut node.node_id)?;
        to_index(&mut node.semantic_type)?;
        for id in &mut node.dependencies {
            to_index(id)?;
        }
        if let Some(id) = node.owner_model.as_mut() {
            to_index(id)?;
        }
        // Walk the document-backed body with the same charged worklist the
        // slot rewrite used. IDs stay in the shared document and resolve
        // through `row` when a reader consumes them.
        let site = Site::new(PackageReadStage::WireChecks, position, "owner_schema");
        resolve_document_body(TermNode::of(body), budget, site)?;
    }
    Ok(())
}

fn resolve_document_body(
    body: super::TermNode<'_>,
    budget: &mut NativeJoinBudget,
    site: Site,
) -> Result<(), ValidationFailure> {
    let mut pending = Vec::new();
    read_charge::push(budget, site, &mut pending, body)?;
    while let Some(value) = pending.pop() {
        if let Some(items) = value.items() {
            read_charge::reserve(budget, site, &mut pending, items.len())?;
            pending.extend(items);
        } else if let Some(members) = value.members() {
            read_charge::reserve(budget, site, &mut pending, members.len())?;
            pending.extend(members.map(|(_, value)| value));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn resolve_value_slots(
    body: &mut serde_json::Value,
    row: &[CheckedOperandEntry],
    budget: &mut NativeJoinBudget,
    site: Site,
) -> Result<(), ValidationFailure> {
    let mut pending = Vec::new();
    read_charge::push(budget, site, &mut pending, body)?;
    while let Some(value) = pending.pop() {
        match value {
            serde_json::Value::Array(items) => {
                read_charge::reserve(budget, site, &mut pending, items.len())?;
                pending.extend(items.iter_mut());
            }
            serde_json::Value::Object(object) => {
                let is_node_id = object.len() == 3
                    && object.contains_key("package")
                    && object.contains_key("ordinal");
                if is_node_id {
                    if let Some(serde_json::Value::Array(path)) = object.get_mut("path") {
                        for segment in path.iter_mut() {
                            let slot = match segment {
                                serde_json::Value::Object(map) if map.len() == 1 => {
                                    map.get("operand").and_then(serde_json::Value::as_u64)
                                }
                                _ => None,
                            };
                            if let (Some(entry), serde_json::Value::Object(map)) = (
                                slot.and_then(|slot| usize::try_from(slot).ok())
                                    .and_then(|slot| row.get(slot)),
                                &mut *segment,
                            ) {
                                // `{"operand": k}` becomes `{"node": entry}` by
                                // renaming the key in the text it already owns.
                                if let Some((mut key, _)) = map.remove_entry("operand") {
                                    key.clear();
                                    key.push_str("node");
                                    match entry {
                                        CheckedOperandEntry::Node(index) => {
                                            map.insert(key, serde_json::Value::from(*index));
                                        }
                                        CheckedOperandEntry::Dependency(operand) => {
                                            map.insert(key, serde_json::Value::from(operand.node));
                                            map.insert(
                                                "package".into(),
                                                serde_json::Value::from(
                                                    operand.package.to_string(),
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                read_charge::reserve(budget, site, &mut pending, object.len())?;
                pending.extend(object.values_mut());
            }
            _ => {}
        }
    }
    Ok(())
}

/// Whether `id` names an operand by slot.
pub(super) fn holds_slot(id: &super::CheckedNodeId) -> bool {
    id.path
        .iter()
        .any(|segment| matches!(segment, CheckedNodePathSegment::Operand(_)))
}

/// The wire form of a resolved graph: each node-held id written with the
/// operand slots its node's row names its operands by.
pub(super) fn slot_graph(graph: &super::CheckedSemanticGraphV2) -> super::CheckedSemanticGraphV2 {
    let mut out = graph.clone();
    for (node, row) in out.nodes.iter_mut().zip(&graph.operand_rows) {
        node.map_node_ids(|id| {
            let held = id.path.iter().any(|segment| {
                matches!(
                    segment,
                    CheckedNodePathSegment::Index(_) | CheckedNodePathSegment::Dependency { .. }
                )
            });
            held.then(|| {
                let mut restored = id.clone();
                restore_slots(&mut restored, row);
                restored
            })
        });
    }
    out
}

/// The inverse of [`resolve_slots`] for one node: write each document index
/// segment of a node-held id as the slot the node's row names it by.
pub(super) fn restore_slots(id: &mut super::CheckedNodeId, row: &[CheckedOperandEntry]) {
    for segment in &mut id.path {
        let slot = row.iter().position(|entry| match (&*segment, entry) {
            (CheckedNodePathSegment::Index(index), CheckedOperandEntry::Node(held)) => {
                index == held
            }
            (
                CheckedNodePathSegment::Dependency { package, node },
                CheckedOperandEntry::Dependency(held),
            ) => package == &held.package && node == &held.node,
            _ => false,
        });
        if let Some(slot) = slot {
            *segment = CheckedNodePathSegment::Operand(
                u64::try_from(slot).expect("row position fits u64"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{contains_inline_node_path, malformed_at, StructuralOrder};
    use crate::checked_package::v2::native_join_budget::{
        NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
    };
    use crate::checked_package::v2::read_charge::Site;
    use crate::checked_package::v2::{CheckedNodeId, CheckedNodePathSegment};
    use ix_trace_rs::trace;
    use quire_canonical::Node;
    use std::sync::Arc;

    fn key(name: &str, operand: Option<&Arc<CheckedNodeId>>, ordinal: u64) -> Arc<CheckedNodeId> {
        let mut path = vec![CheckedNodePathSegment::Name(name.into())];
        path.extend(operand.map(|operand| CheckedNodePathSegment::Node(Arc::clone(operand))));
        Arc::new(CheckedNodeId {
            package: "p".into(),
            path,
            ordinal,
        })
    }

    /// `length` keys, each naming the one before it, with `salt` as the last
    /// ordinal so that two chains differ only at their tops.
    fn chain(length: usize, salt: u64) -> Vec<Arc<CheckedNodeId>> {
        let mut keys: Vec<Arc<CheckedNodeId>> = Vec::with_capacity(length);
        for position in 0..length {
            let ordinal = if position + 1 == length { salt } else { 0 };
            let next = key("n", keys.last(), ordinal);
            keys.push(next);
        }
        keys
    }

    fn node_index_site() -> Site {
        Site::new(PackageReadStage::NodeIndex, 0, "node_index")
    }

    /// The order ranks keys as `CheckedNodeId::cmp` does, gives equal keys of
    /// two documents one token, and finds a resolved ID without allocating.
    ///
    /// Trace: TC-472, FR-038-AC-500.
    #[test]
    fn structural_order_ranks_like_cmp_and_shares_tokens_across_documents() {
        let first = chain(3000, 1);
        let second = chain(3000, 1);
        let other = chain(3000, 2);
        let order = StructuralOrder::build_unmetered(&[&first, &second, &other])
            .expect("every operand is a listed key");
        for (a, b) in first.iter().zip(&second) {
            assert!(!Arc::ptr_eq(a, b));
            assert_eq!(order.token(a), order.token(b));
            assert_eq!(order.rank(a), order.rank(b));
        }
        for left in first.iter().step_by(97) {
            for right in other.iter().step_by(211) {
                assert_eq!(
                    left.as_ref().cmp(right.as_ref()),
                    order.rank(left).cmp(&order.rank(right))
                );
            }
        }
        // The two tops differ only in ordinal, deep below the equal spine.
        let (top, rival) = (first.last().unwrap(), other.last().unwrap());
        assert_eq!(top.cmp(rival), order.rank(top).cmp(&order.rank(rival)));
        assert_ne!(order.token(top), order.token(rival));
        let copy = CheckedNodeId::clone(&second[10]);
        assert_eq!(order.find(&copy), order.token(&first[10]));
        let absent = CheckedNodeId {
            package: "q".into(),
            path: vec![CheckedNodePathSegment::Name("n".into())],
            ordinal: 0,
        };
        assert_eq!(order.find(&absent), None);
    }

    /// The order's tables are charged exactly: the measured requirement
    /// admits and one byte below is denied at the node-index locus.
    ///
    /// Trace: TC-472, FR-038-AC-500, FR-358-AC-13.
    #[test]
    fn structural_order_is_charged_to_the_byte_at_the_node_index_locus() {
        let keys = chain(200, 0);
        let mut measure = NativeJoinBudget::unlimited();
        assert!(
            StructuralOrder::build(&[&keys], &mut measure, node_index_site())
                .expect("charged")
                .is_some()
        );
        let required = measure.read_storage_consumed();
        let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
        assert!(
            StructuralOrder::build(&[&keys], &mut exact, node_index_site())
                .expect("exact allowance admits")
                .is_some()
        );
        assert_eq!(exact.read_storage_consumed(), required);
        let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required - 1);
        assert!(StructuralOrder::build(&[&keys], &mut short, node_index_site()).is_err());
        assert_eq!(
            short.first_unpaid_locus(),
            Some(NativeJoinLocus {
                source: NativeJoinLocusSource::PackageRead(PackageReadStage::NodeIndex, 0),
                form: "node_index",
            })
        );
    }

    fn pointer_document() -> quire_canonical::Document {
        quire_canonical::read(br#"{"a":[1,{"b":2,"c~":[]}],"d":{"e":3}}"#, 1 << 20)
            .expect("document")
    }

    #[trace("TC-472", "FR-038-AC-501", "FR-038-AC-510", "FR-038-AC-511")]
    #[test]
    fn located_faults_pay_exact_pointer_record_and_location_before_allocating() {
        use super::super::super::common::ValidationFailure;
        use super::super::CheckedPackagePendingOutcome;
        let document = pointer_document();
        let mut pending = vec![document.root()];
        while let Some(value) = pending.pop() {
            let run = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let actual = allocation_counter::measure(|| {
                    result = Some(malformed_at(&document, value, &mut budget));
                });
                (result.unwrap(), actual.bytes_total, budget)
            };
            let (result, actual, mut budget) = run(u64::MAX);
            let ValidationFailure::Refused(refusal) = &result else {
                panic!("located fault")
            };
            assert_eq!(
                refusal.code,
                super::CheckedPackageRefusalCode::MalformedWire
            );
            assert_eq!(refusal.cause, None);
            assert_eq!(refusal.locus, None);
            let required = budget.read_storage_consumed();
            assert_eq!(actual, required);
            assert_eq!(
                budget.take_paid(),
                Some(CheckedPackagePendingOutcome::Refused(
                    super::CheckedPackageRefusalCode::MalformedWire
                ))
            );
            assert_eq!(run(required).0, result);
            for limit in [0, required - 1] {
                let (result, actual, budget) = run(limit);
                assert!(matches!(result, ValidationFailure::Budget(_)));
                assert_eq!(actual, 0, "no part of a denied diagnostic is allocated");
                assert_eq!(budget.read_storage_consumed(), 0);
                assert_eq!(budget.first_unpaid_read_storage(), Some(required as u128));
                assert_eq!(
                    budget.first_unpaid_locus(),
                    Some(NativeJoinLocus {
                        source: NativeJoinLocusSource::PackageRead(PackageReadStage::NodeIndex, 0),
                        form: "diagnostic",
                    })
                );
            }
            match value.node() {
                Node::Object(members) => pending.extend(members.map(|(_, child)| child)),
                Node::Array(items) => pending.extend(items),
                _ => {}
            }
        }
    }

    #[cfg(feature = "test-support")]
    #[trace("TC-472", "FR-038-AC-496", "FR-038-AC-501")]
    #[test]
    fn located_fault_replays_keep_all_prior_caller_counters_and_first_denial() {
        use super::super::native_join_budget::NativeJoinChargeKind;
        let document = pointer_document();
        let Node::Object(mut root) = document.root().node() else {
            panic!("object fixture")
        };
        let array = root.find(|(name, _)| *name == "a").unwrap().1;
        let Node::Array(mut items) = array.node() else {
            panic!("array fixture")
        };
        let target = items.next().unwrap();
        let mut measured = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let _ = malformed_at(&document, target, &mut measured);
        let required = measured.read_storage_consumed();
        let mut caller = NativeJoinBudget::new(100, 100).with_read_storage_limit(7 + 2 * required);
        let before = caller.begin();
        assert!(caller.charge(NativeJoinChargeKind::Work, 3));
        assert!(caller.charge(NativeJoinChargeKind::Storage, 2));
        super::read_charge::charge(&mut caller, super::site(9), 7).unwrap();
        caller.finish_read(before, 0);
        let original_work_storage = caller.consumed();
        let original_delta = caller.last_success_delta();
        let original_read_delta = caller.last_success_read_storage_delta();
        let mut replay = caller.read_proof_snapshot();
        for step in 1..=2 {
            let mut result = None;
            let actual = allocation_counter::measure(|| {
                result = Some(malformed_at(&document, target, &mut caller));
            });
            let other = malformed_at(&document, target, &mut replay);
            assert_eq!(result.unwrap(), other);
            assert_eq!(
                caller, replay,
                "every captured budget field replays equally"
            );
            assert_eq!(actual.bytes_total, required);
            assert_eq!(caller.read_storage_consumed(), 7 + step * required);
            assert_eq!(caller.consumed(), original_work_storage);
            assert_eq!(caller.last_success_delta(), original_delta);
            assert_eq!(
                caller.last_success_read_storage_delta(),
                original_read_delta
            );
            assert_eq!(caller.first_unpaid_read_storage(), None);
            assert_eq!(caller.take_paid(), replay.take_paid());
        }
        let mut result = None;
        let actual = allocation_counter::measure(|| {
            result = Some(malformed_at(&document, target, &mut caller));
        });
        assert!(matches!(
            result.unwrap(),
            super::ValidationFailure::Budget(_)
        ));
        assert_eq!(actual.bytes_total, 0);
        assert_eq!(caller.first_unpaid_read_storage(), Some(required as u128));
        let denied = caller.read_proof_snapshot();
        let _ = malformed_at(&document, target, &mut caller);
        assert_eq!(
            caller, denied,
            "retry cannot replace the first unpaid charge or clear counters"
        );
    }

    #[trace("TC-472", "FR-038-AC-501", "FR-038-AC-511")]
    #[test]
    fn key_range_and_cycle_faults_keep_the_first_operand_and_pay_their_diagnostic() {
        use super::super::stage_storage::Path;
        for paths in [
            vec![vec![CheckedNodePathSegment::Index(1)]],
            vec![
                vec![CheckedNodePathSegment::Index(1)],
                vec![CheckedNodePathSegment::Index(0)],
            ],
        ] {
            let ids: Vec<_> = paths
                .into_iter()
                .map(|path| CheckedNodeId {
                    package: "p".into(),
                    path,
                    ordinal: 0,
                })
                .collect();
            let run = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let actual = allocation_counter::measure(|| {
                    result = Some(super::rebuild_keys_with_failure(
                        ids.len(),
                        |position| &ids[position],
                        |position, slot, budget| {
                            super::malformed_path(
                                budget,
                                position,
                                Path::root()
                                    .key("semantic_graph")
                                    .key("nodes")
                                    .index(position)
                                    .key("node_id")
                                    .key("path")
                                    .index(slot)
                                    .key("operand"),
                            )
                        },
                        &mut budget,
                    ));
                });
                (result.unwrap(), actual.bytes_total, budget)
            };
            let (result, actual, budget) = run(u64::MAX);
            let Err(super::ValidationFailure::Refused(refusal)) = &result else {
                panic!("bad operand")
            };
            assert_eq!(
                refusal.code,
                super::CheckedPackageRefusalCode::MalformedWire
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().unwrap().as_str(),
                "/semantic_graph/nodes/0/node_id/path/0/operand"
            );
            assert_eq!(actual, budget.read_storage_consumed());
            assert_eq!(run(actual).0, result);
            let (result, actual, budget) = run(actual - 1);
            assert!(matches!(result, Err(super::ValidationFailure::Budget(_))));
            assert_eq!(actual, budget.read_storage_consumed());
            assert_eq!(budget.first_unpaid_locus().unwrap().form, "diagnostic");
        }
    }

    /// The pointer to every value of a document is found from its id alone.
    ///
    /// Trace: TC-472, FR-038-AC-500.
    #[test]
    fn locate_finds_every_value_by_id() {
        let document = pointer_document();
        let mut found = Vec::new();
        let mut pending = vec![document.root()];
        while let Some(value) = pending.pop() {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
            let super::ValidationFailure::Refused(refusal) =
                malformed_at(&document, value, &mut budget)
            else {
                panic!("affordable located refusal")
            };
            found.push(refusal.loci[0].path.as_ref().unwrap().as_str().to_owned());
            match value.node() {
                Node::Object(members) => pending.extend(members.map(|(_, child)| child)),
                Node::Array(items) => pending.extend(items),
                _ => {}
            }
        }
        found.sort();
        assert_eq!(
            found,
            ["", "/a", "/a/0", "/a/1", "/a/1/b", "/a/1/c~0", "/d", "/d/e"]
        );
    }

    /// Trace: TC-048, FR-038-AC-74, FR-038-AC-377.
    #[test]
    fn envelope_scan_distinguishes_index_from_inline_and_declared_reference() {
        assert!(!contains_inline_node_path(
            br#"{"node_id":{"ordinal":0,"package":"p","path":["x",{"node":1}]}}"#
        ));
        assert!(contains_inline_node_path(
            br#"{"node_id":{"ordinal":0,"package":"p","path":["x",{"ordinal":0,"package":"p","path":["y"]}]}}"#
        ));
        assert!(contains_inline_node_path(
            br#"{"body":{"node":{"ordinal":0,"package":"p","path":[{"node":1}]},"package":"q","term":"dependency_reference"}}"#
        ));
        assert!(!contains_inline_node_path(
            br#"{"details":{"ordinal":0,"package":"p","path":["x"]}}"#
        ));
    }
}

#[cfg(test)]
mod imported_body_storage_tests {
    use super::*;
    use crate::checked_package::v2::{CheckedJson, NativeJoinBudget, TermNode, WorkMeter};
    use ix_trace_rs::trace;
    #[test]
    #[trace("TC-472", "FR-038-AC-511")]
    fn imported_document_comparison_prepays_operand_and_scratch_storage() {
        let body = serde_json::json!({"term":"aggregate","members":[{"term":"reference","target":{"package":"p","path":[{"operand":0}],"ordinal":0}}]});
        let left = CheckedJson::from_value(&body).expect("document");
        let right = CheckedJson::from_value(&body).expect("document");
        let make_keys = || {
            let mut node = Arc::new(super::super::CheckedNodeId {
                package: "p".into(),
                path: vec![CheckedNodePathSegment::Name("target".into())],
                ordinal: 0,
            });
            for _ in 0..512 {
                node = Arc::new(super::super::CheckedNodeId {
                    package: "p".into(),
                    path: vec![CheckedNodePathSegment::Node(node)],
                    ordinal: 0,
                });
            }
            [node]
        };
        let left_keys = make_keys();
        let right_keys = make_keys();
        assert!(!Arc::ptr_eq(&left_keys[0], &right_keys[0]));
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let allocated = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(0, "import_body");
                result = Some(equivalent_values(
                    TermNode::of(&left),
                    TermNode::of(&right),
                    &left_keys,
                    &right_keys,
                    &[crate::CheckedOperandEntry::Node(0)],
                    &[crate::CheckedOperandEntry::Node(0)],
                    &mut meter,
                ));
            });
            (result.expect("comparison"), budget, allocated.bytes_total)
        };
        let (result, budget, allocated) = run(u64::MAX);
        assert_eq!(result.ok(), Some(true));
        assert_eq!(allocated, budget.read_storage_consumed());
        let exact = budget.read_storage_consumed();
        let (result, budget, allocated) = run(exact);
        assert_eq!(result.ok(), Some(true));
        assert_eq!(allocated, exact);
        assert_eq!(budget.read_storage_consumed(), exact);
        let (result, budget, allocated) = run(exact - 1);
        assert!(result.is_err());
        assert!(budget.first_unpaid_read_storage().is_some());
        assert_eq!(allocated, budget.read_storage_consumed());
    }
}
