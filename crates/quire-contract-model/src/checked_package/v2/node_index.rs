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
use super::{CheckedNodePathSegment, RawCheckedPackageWireV2, ValidationFailure};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
use quire_canonical::{Document, Items, Members, Node, NodeRef};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum TokenSegment {
    Name(Box<str>),
    Operand(usize),
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct TokenDescriptor {
    package: Box<str>,
    path: Vec<TokenSegment>,
    ordinal: u64,
}

/// Assign compact equality tokens to resolved keys from multiple documents.
/// Descriptors compare only immediate names and operand tokens, so a long
/// derived chain never triggers repeated deep structural comparisons.
pub(super) struct StructuralInterner {
    descriptors: BTreeMap<TokenDescriptor, usize>,
    pointers: BTreeMap<*const super::CheckedNodeId, usize>,
}

impl StructuralInterner {
    pub(super) fn new() -> Self {
        Self {
            descriptors: BTreeMap::new(),
            pointers: BTreeMap::new(),
        }
    }

    pub(super) fn tokens(&mut self, keys: &[Arc<super::CheckedNodeId>]) -> Option<Vec<usize>> {
        enum Task<'a> {
            Visit(&'a super::CheckedNodeId),
            Finish(&'a super::CheckedNodeId),
        }
        let mut output = Vec::with_capacity(keys.len());
        for root in keys {
            let mut pending = vec![Task::Visit(root)];
            while let Some(task) = pending.pop() {
                match task {
                    Task::Visit(node) => {
                        if self.pointers.contains_key(&std::ptr::from_ref(node)) {
                            continue;
                        }
                        pending.push(Task::Finish(node));
                        for segment in node.path.iter().rev() {
                            if let CheckedNodePathSegment::Node(operand) = segment {
                                pending.push(Task::Visit(operand));
                            }
                        }
                    }
                    Task::Finish(node) => {
                        let mut path = Vec::with_capacity(node.path.len());
                        for segment in &node.path {
                            path.push(match segment {
                                CheckedNodePathSegment::Name(name) => {
                                    TokenSegment::Name(name.clone())
                                }
                                CheckedNodePathSegment::Node(operand) => TokenSegment::Operand(
                                    *self.pointers.get(&Arc::as_ptr(operand))?,
                                ),
                                CheckedNodePathSegment::Index(_)
                                | CheckedNodePathSegment::Operand(_) => return None,
                            });
                        }
                        let descriptor = TokenDescriptor {
                            package: node.package.clone(),
                            path,
                            ordinal: node.ordinal,
                        };
                        let next = self.descriptors.len();
                        let token = *self.descriptors.entry(descriptor).or_insert(next);
                        self.pointers.insert(std::ptr::from_ref(node), token);
                    }
                }
            }
            output.push(*self.pointers.get(&Arc::as_ptr(root))?);
        }
        Some(output)
    }
}

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

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

/// The charge site of the node-index stage at graph node `index`.
fn site(index: usize) -> Site {
    Site::new(PackageReadStage::NodeIndex, index, "node_index")
}

/// The RFC 6901 pointer to the value `target` of `document`. Only a refusal
/// asks for one, so no walk keeps a pointer beside its values.
///
/// A document numbers its values children first, in the order they were read,
/// so the first child of a container whose id is not below `target` holds it.
fn locate(document: &Document, target: NodeRef<'_>) -> JsonPointer {
    let mut pointer = JsonPointer::root();
    let mut value = document.root();
    let target = target.id();
    while value.id() != target {
        match value.node() {
            Node::Object(mut members) => {
                let Some((name, child)) = members.find(|(_, child)| child.id() >= target) else {
                    break;
                };
                pointer = pointer.key(name);
                value = child;
            }
            Node::Array(items) => {
                let Some((position, child)) =
                    items.enumerate().find(|(_, child)| child.id() >= target)
                else {
                    break;
                };
                pointer = pointer.index(position);
                value = child;
            }
            Node::Null | Node::Bool(_) | Node::Number(_) | Node::String(_) => break,
        }
    }
    pointer
}

/// The refusal at the value `target` of `document`.
fn malformed_at(document: &Document, target: NodeRef<'_>) -> ValidationFailure {
    malformed(locate(document, target))
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
    value: u64,
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
    left: &serde_json::Value,
    right: &serde_json::Value,
    left_keys: &[Arc<super::CheckedNodeId>],
    right_keys: &[Arc<super::CheckedNodeId>],
) -> bool {
    let mut pending = vec![(left, right)];
    while let Some((left, right)) = pending.pop() {
        match (left, right) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                // FR-375 external keys are the exception to document-local
                // index scope: their `node` is scoped by the outer package.
                // Both documents select that dependency under the same key,
                // so compare its wire ID directly instead of resolving it in
                // either containing document's graph.
                if a.len() == 2 && a.contains_key("package") && a.contains_key("node") {
                    if a != b {
                        return false;
                    }
                    continue;
                }
                let node_shape = |map: &serde_json::Map<String, serde_json::Value>| {
                    map.len() == 3
                        && map.contains_key("package")
                        && map.contains_key("path")
                        && map.contains_key("ordinal")
                };
                if node_shape(a) || node_shape(b) {
                    let Some(a) = super::CheckedNodeId::from_value(left) else {
                        return false;
                    };
                    let Some(b) = super::CheckedNodeId::from_value(right) else {
                        return false;
                    };
                    if resolve_id(&a, left_keys) != resolve_id(&b, right_keys) {
                        return false;
                    }
                } else {
                    if a.len() != b.len() {
                        return false;
                    }
                    for (name, value) in a {
                        let Some(other) = b.get(name) else {
                            return false;
                        };
                        pending.push((value, other));
                    }
                }
            }
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                pending.extend(a.iter().zip(b));
            }
            (a, b) if a == b => {}
            _ => return false,
        }
    }
    true
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
    // The graph position a segment names, or `None` for a segment that is
    // not an operand reference of this table.
    let target_of = |segment: &CheckedNodePathSegment| -> Option<Option<usize>> {
        match segment {
            CheckedNodePathSegment::Name(_) | CheckedNodePathSegment::Node(_) => Some(None),
            CheckedNodePathSegment::Index(index) => Some(Some(usize::try_from(*index).ok()?)),
            CheckedNodePathSegment::Operand(_) => None,
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
                Some(Some(_)) | None => return Err(refuse(pointer(position, slot))),
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
                return Err(refuse(pointer(node, slot)));
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
                CheckedNodePathSegment::Index(_) | CheckedNodePathSegment::Operand(_) => {
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
    fn check_dependency_reference(&self, members: &Members<'d>) -> Result<(), ValidationFailure> {
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
            return Err(malformed_at(self.document, node));
        };
        let Some((_, path)) = node_members.clone().find(|(name, _)| *name == "path") else {
            return Ok(());
        };
        let Node::Array(segments) = path.node() else {
            return Err(malformed_at(self.document, path));
        };
        for segment in segments {
            if matches!(segment.node(), Node::Object(_)) {
                return Err(malformed_at(self.document, segment));
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
            return Err(malformed_at(self.document, path));
        };
        let holder = match walk.scope {
            Scope::Node(holder) => Some(holder),
            _ => None,
        };
        for segment in segments {
            let Node::Object(mut object) = segment.node() else {
                continue;
            };
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
                return Err(malformed_at(self.document, segment));
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
            let Some(value) = as_u64(child) else {
                return Err(malformed_at(document, child));
            };
            read_charge::push(
                collector.budget,
                site(0),
                &mut collector.row_sites,
                RowSite { id: child, value },
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
    let graph_at = || JsonPointer::root().key("semantic_graph");
    // 1. Rows.
    if rows.len() != node_count {
        return Err(malformed(graph_at().key("operand_rows")));
    }
    let (sites, row_sites) = collect_sites(document, node_count, evidence, budget)?;
    // 2. Placement. A foreign key's segments name its dependency's graph.
    if let Some(site) = sites
        .iter()
        .find(|site| site.foreign_graph.is_none() && site.slot != site.holder.is_some())
    {
        return Err(malformed_at(document, site.id));
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
    let bad_entry = row_sites.iter().find_map(|site| {
        usize::try_from(site.value)
            .ok()
            .is_none_or(|value| value >= node_count)
            .then_some(site.id)
    });
    if let Some(at) = [out_of_range, bad_entry]
        .into_iter()
        .flatten()
        .min_by_key(|at| at.id())
    {
        return Err(malformed_at(document, at));
    }
    // 4. Numbering by first occurrence, node by node in graph order. The
    // sites are in document order, so a node's slots are consecutive and in
    // ascending node order.
    let longest_row = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut entries: Vec<(u64, usize)> = read_charge::with_capacity(budget, site(0), longest_row)?;
    let mut slots = sites
        .iter()
        .filter(|site| site.slot && site.holder.is_some() && site.foreign_graph.is_none())
        .peekable();
    for (node, row) in rows.iter().enumerate() {
        let mut named = 0u64;
        while let Some(site) = slots.next_if(|site| site.holder == Some(node)) {
            if site.value > named {
                return Err(malformed_at(document, site.id));
            }
            named = named.max(site.value + 1);
        }
        // The first row position that is unnamed or repeats an earlier entry.
        entries.clear();
        entries.extend(row.iter().copied().zip(0..));
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
            return Err(malformed(
                graph_at().key("operand_rows").index(node).index(position),
            ));
        }
    }
    drop(entries);
    drop(sites);
    drop(row_sites);
    // 5. Cycle, then the keys themselves.
    let keys = rebuild_keys(
        node_count,
        |position| &wire.semantic_graph.nodes[position].node_id,
        |position, slot| {
            graph_at()
                .key("nodes")
                .index(position)
                .key("node_id")
                .key("path")
                .index(slot)
                .key("operand")
        },
        malformed,
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
    budget: &mut NativeJoinBudget,
) -> Result<(), ValidationFailure> {
    let graph = &mut wire.semantic_graph;
    let rows = &graph.operand_rows;
    for (position, (node, row)) in graph.nodes.iter_mut().zip(rows).enumerate() {
        let to_index = |id: &mut super::CheckedNodeId| {
            for segment in &mut id.path {
                if let CheckedNodePathSegment::Operand(slot) = segment {
                    if let Some(entry) = usize::try_from(*slot).ok().and_then(|slot| row.get(slot))
                    {
                        *segment = CheckedNodePathSegment::Index(*entry);
                    }
                }
            }
        };
        to_index(&mut node.node_id);
        to_index(&mut node.semantic_type);
        node.dependencies.iter_mut().for_each(to_index);
        if let Some(id) = node.owner_model.as_mut() {
            to_index(id);
        }
        let site = Site::new(PackageReadStage::WireChecks, position, "owner_schema");
        resolve_value_slots(&mut node.body, row, budget, site)?;
    }
    Ok(())
}

pub(super) fn resolve_value_slots(
    body: &mut serde_json::Value,
    row: &[u64],
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
                                    map.insert(key, serde_json::Value::from(*entry));
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
            let held = id
                .path
                .iter()
                .any(|segment| matches!(segment, CheckedNodePathSegment::Index(_)));
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
pub(super) fn restore_slots(id: &mut super::CheckedNodeId, row: &[u64]) {
    for segment in &mut id.path {
        if let CheckedNodePathSegment::Index(index) = segment {
            if let Some(slot) = row.iter().position(|entry| entry == index) {
                *segment = CheckedNodePathSegment::Operand(
                    u64::try_from(slot).expect("a row position fits in u64"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{contains_inline_node_path, locate, StructuralOrder};
    use crate::checked_package::v2::native_join_budget::{
        NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
    };
    use crate::checked_package::v2::read_charge::Site;
    use crate::checked_package::v2::{CheckedNodeId, CheckedNodePathSegment};
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

    /// The pointer to every value of a document is found from its id alone.
    ///
    /// Trace: TC-472, FR-038-AC-500.
    #[test]
    fn locate_finds_every_value_by_id() {
        let document = quire_canonical::read(br#"{"a":[1,{"b":2,"c~":[]}],"d":{"e":3}}"#, 1 << 20)
            .expect("document");
        let mut found = Vec::new();
        let mut pending = vec![document.root()];
        while let Some(value) = pending.pop() {
            found.push(locate(&document, value).as_str().to_owned());
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
