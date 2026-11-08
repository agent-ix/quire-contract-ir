//! FR-322 operand-index checks before semantic-graph admission.
//!
//! The wire is walked through borrowed source spans so an index in any node
//! reference has a source pointer without building a second document tree.
//! Source spans avoid serde_json recursion at deep sibling terms.

use super::{CheckedNodePathSegment, RawCheckedPackageWireV2, ValidationFailure};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
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

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

/// One operand segment of the document, in canonical byte order by `offset`.
struct SegmentSite {
    offset: usize,
    value: u64,
    /// An operand slot (`{"operand": k}`) rather than an index segment.
    slot: bool,
    /// The node whose members hold this segment; `None` in a document-held id.
    holder: Option<usize>,
    /// Inside a foreign `{package, node}` key, which names a dependency's
    /// graph of this many nodes and takes no part in the placement check.
    foreign_graph: Option<usize>,
    at: JsonPointer,
}

/// One entry of an operand row.
struct RowSite {
    offset: usize,
    value: u64,
    at: JsonPointer,
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

fn object_members(raw: &[u8]) -> Option<Vec<(String, &[u8])>> {
    let mut members = Vec::new();
    let mut at = 1;
    while raw.get(at) != Some(&b'}') {
        let end = super::intake::value_end(raw, at)?;
        let name: String = serde_json::from_slice(raw.get(at..end)?).ok()?;
        at = end;
        if raw.get(at) != Some(&b':') {
            return None;
        }
        at += 1;
        let end = super::intake::value_end(raw, at)?;
        members.push((name, raw.get(at..end)?));
        at = end;
        match raw.get(at)? {
            b',' => at += 1,
            b'}' => break,
            _ => return None,
        }
    }
    Some(members)
}

fn array_items(raw: &[u8]) -> Option<Vec<&[u8]>> {
    let mut items = Vec::new();
    let mut at = 1;
    while raw.get(at) != Some(&b']') {
        let end = super::intake::value_end(raw, at)?;
        items.push(raw.get(at..end)?);
        at = end;
        match raw.get(at)? {
            b',' => at += 1,
            b']' => break,
            _ => return None,
        }
    }
    Some(items)
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

/// Resolve graph-local node operands for either the checked V2 envelope or
/// its FR-330 ContractPackage projection. The caller owns refusal coding.
/// Each node-held operand slot resolves through the node's operand row.
#[ix_trace_rs::string_edge]
pub(super) fn rebuild_graph_keys(
    nodes: &[super::RawSemanticNodeV2],
    rows: Option<&[Vec<u64>]>,
    refuse: impl Fn(JsonPointer) -> ValidationFailure,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    let operand = if rows.is_some() { "operand" } else { "node" };
    rebuild_keys(
        nodes
            .iter()
            .map(|node| &node.node_id)
            .collect::<Vec<_>>()
            .as_slice(),
        rows,
        |position, slot| {
            JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position)
                .key("node_id")
                .key("path")
                .index(slot)
                .key(operand)
        },
        refuse,
    )
}

/// Resolve one document-local table of IDs without recursive key construction.
/// With `rows`, the IDs are node-held: they hold operand slots read through
/// the operand row of the node at their position. Without, they are
/// document-held and hold index segments. The caller supplies the
/// table-specific pointer for an operand.
pub(super) fn rebuild_keys(
    ids: &[&super::CheckedNodeId],
    rows: Option<&[Vec<u64>]>,
    pointer: impl Fn(usize, usize) -> JsonPointer,
    refuse: impl Fn(JsonPointer) -> ValidationFailure,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    let node_count = ids.len();
    // The graph position a segment names, or `None` for a segment that is
    // not an operand reference of this table's kind.
    let target_of = |position: usize, segment: &CheckedNodePathSegment| -> Option<Option<usize>> {
        match (rows, segment) {
            (_, CheckedNodePathSegment::Name(_) | CheckedNodePathSegment::Node(_)) => Some(None),
            (None, CheckedNodePathSegment::Index(index)) => {
                Some(Some(usize::try_from(*index).ok()?))
            }
            (Some(rows), CheckedNodePathSegment::Operand(slot)) => {
                let entry = rows.get(position)?.get(usize::try_from(*slot).ok()?)?;
                Some(Some(usize::try_from(*entry).ok()?))
            }
            (None, CheckedNodePathSegment::Operand(_))
            | (Some(_), CheckedNodePathSegment::Index(_)) => None,
        }
    };
    let mut edges: Vec<Vec<(usize, usize)>> = Vec::with_capacity(node_count);
    for (position, id) in ids.iter().enumerate() {
        let mut outgoing = Vec::new();
        for (slot, segment) in id.path.iter().enumerate() {
            match target_of(position, segment) {
                Some(None) => {}
                Some(Some(target)) if target < node_count => outgoing.push((slot, target)),
                Some(Some(_)) | None => return Err(refuse(pointer(position, slot))),
            }
        }
        edges.push(outgoing);
    }
    // Kosaraju's two heap-backed passes make the cycle decision linear in
    // nodes plus operand segments, independent of graph order and chain depth.
    let mut seen = vec![false; node_count];
    let mut order = Vec::with_capacity(node_count);
    for start in 0..node_count {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut stack = vec![(start, 0usize)];
        while let Some((node, cursor)) = stack.pop() {
            if let Some((_, target)) = edges[node].get(cursor) {
                stack.push((node, cursor + 1));
                if !seen[*target] {
                    seen[*target] = true;
                    stack.push((*target, 0));
                }
            } else {
                order.push(node);
            }
        }
    }
    let mut reverse = vec![Vec::new(); node_count];
    for (node, outgoing) in edges.iter().enumerate() {
        for (_, target) in outgoing {
            reverse[*target].push(node);
        }
    }
    let mut component = vec![usize::MAX; node_count];
    for &start in order.iter().rev() {
        if component[start] != usize::MAX {
            continue;
        }
        let mut stack = vec![start];
        component[start] = start;
        while let Some(node) = stack.pop() {
            for &previous in &reverse[node] {
                if component[previous] == usize::MAX {
                    component[previous] = start;
                    stack.push(previous);
                }
            }
        }
    }
    for (node, outgoing) in edges.iter().enumerate() {
        for &(slot, target) in outgoing {
            if component[node] == component[target] {
                return Err(refuse(pointer(node, slot)));
            }
        }
    }
    let mut keys: Vec<Option<Arc<super::CheckedNodeId>>> = vec![None; node_count];
    for node in order {
        let source = ids[node];
        let path = source
            .path
            .iter()
            .map(|segment| match segment {
                CheckedNodePathSegment::Name(name) => CheckedNodePathSegment::Name(name.clone()),
                CheckedNodePathSegment::Node(key) => CheckedNodePathSegment::Node(Arc::clone(key)),
                CheckedNodePathSegment::Index(_) | CheckedNodePathSegment::Operand(_) => {
                    let target = target_of(node, segment)
                        .flatten()
                        .expect("operand was checked in bounds");
                    CheckedNodePathSegment::Node(Arc::clone(
                        keys[target].as_ref().expect("target key built first"),
                    ))
                }
            })
            .collect();
        keys[node] = Some(Arc::new(super::CheckedNodeId {
            package: source.package.clone(),
            path,
            ordinal: source.ordinal,
        }));
    }
    Ok(keys
        .into_iter()
        .map(|key| key.expect("each graph node key built"))
        .collect())
}

#[ix_trace_rs::string_edge]
/// Collect the operand segments and operand row entries of `bytes` in
/// document order. A segment object is `{"operand": k}` or `{"node": i}` as
/// the sole member of a path item of a node ID.
fn collect_sites(
    bytes: &[u8],
    node_count: usize,
    evidence: &CheckedPackageEvidence,
) -> Result<(Vec<SegmentSite>, Vec<RowSite>), ValidationFailure> {
    let mut pending = vec![(
        bytes,
        JsonPointer::root(),
        Walk {
            graph_nodes: node_count,
            scope: Scope::Root,
            foreign_graph: None,
        },
    )];
    let mut sites = Vec::new();
    let mut row_sites = Vec::new();
    let offset_of = |part: &[u8]| (part.as_ptr() as usize).saturating_sub(bytes.as_ptr() as usize);
    while let Some((raw, at, walk)) = pending.pop() {
        match raw.first() {
            Some(b'{') => {
                let members = object_members(raw).ok_or_else(|| malformed(at.clone()))?;
                if members
                    .iter()
                    .any(|(name, value)| name == "term" && *value == b"\"dependency_reference\"")
                {
                    if let Some((_, node)) = members.iter().find(|(name, _)| name == "node") {
                        let node_members = object_members(node)
                            .ok_or_else(|| malformed(at.clone().key("node")))?;
                        if let Some((_, path)) =
                            node_members.iter().find(|(name, _)| name == "path")
                        {
                            let segments = array_items(path)
                                .ok_or_else(|| malformed(at.clone().key("node").key("path")))?;
                            for (slot, segment) in segments.iter().enumerate() {
                                if segment.starts_with(b"{") {
                                    return Err(malformed(
                                        at.clone().key("node").key("path").index(slot),
                                    ));
                                }
                            }
                        }
                    }
                }
                if members.len() == 3
                    && ["package", "path", "ordinal"]
                        .iter()
                        .all(|name| members.iter().any(|(member, _)| member == name))
                {
                    let path = members
                        .iter()
                        .find(|(name, _)| name == "path")
                        .expect("checked member present")
                        .1;
                    let segments =
                        array_items(path).ok_or_else(|| malformed(at.clone().key("path")))?;
                    for (slot, segment) in segments.into_iter().enumerate() {
                        if !segment.starts_with(b"{") {
                            continue;
                        }
                        let object = object_members(segment)
                            .ok_or_else(|| malformed(at.clone().key("path").index(slot)))?;
                        if object.len() != 1 || !matches!(object[0].0.as_str(), "node" | "operand")
                        {
                            continue;
                        }
                        let (name, member) = (&object[0].0, object[0].1);
                        let value: u64 = serde_json::from_slice(member)
                            .map_err(|_| malformed(at.clone().key("path").index(slot)))?;
                        sites.push(SegmentSite {
                            offset: offset_of(member),
                            value,
                            slot: name == "operand",
                            holder: match walk.scope {
                                Scope::Node(holder) => Some(holder),
                                _ => None,
                            },
                            foreign_graph: walk.foreign_graph,
                            at: at.clone().key("path").index(slot).key(name),
                        });
                    }
                }
                // FR-375's external key is scoped by its outer package,
                // regardless of the package named inside its node ID.
                let is_external_key = members.len() == 2
                    && members.iter().any(|(name, _)| name == "node")
                    && members.iter().any(|(name, _)| name == "package");
                let external_graph_nodes = if is_external_key {
                    members
                        .iter()
                        .find(|(name, _)| name == "package")
                        .and_then(|(_, raw)| serde_json::from_slice::<String>(raw).ok())
                        .and_then(|package| evidence.dependency_package(&package))
                        .map(|package| package.resolved_graph().nodes.len())
                } else {
                    None
                };
                for (name, child) in members.into_iter().rev() {
                    let mut child_walk = walk;
                    child_walk.scope = match (walk.scope, name.as_str()) {
                        (Scope::Root, "semantic_graph") => Scope::Graph,
                        (Scope::Graph, "nodes") => Scope::Nodes,
                        (Scope::Graph, "operand_rows") => Scope::Rows,
                        (Scope::Node(holder), _) => Scope::Node(holder),
                        _ => Scope::Other,
                    };
                    if name == "node" && is_external_key {
                        let nodes = external_graph_nodes.unwrap_or(usize::MAX);
                        child_walk.graph_nodes = nodes;
                        child_walk.foreign_graph = Some(nodes);
                    }
                    pending.push((child, at.clone().key(&name), child_walk));
                }
            }
            Some(b'[') => {
                let items = array_items(raw).ok_or_else(|| malformed(at.clone()))?;
                for (position, child) in items.into_iter().enumerate().rev() {
                    let mut child_walk = walk;
                    child_walk.scope = match walk.scope {
                        Scope::Nodes => Scope::Node(position),
                        Scope::Rows => Scope::Row,
                        Scope::Node(holder) => Scope::Node(holder),
                        _ => Scope::Other,
                    };
                    if let Scope::Row = walk.scope {
                        let value: u64 = serde_json::from_slice(child)
                            .map_err(|_| malformed(at.clone().index(position)))?;
                        row_sites.push(RowSite {
                            offset: offset_of(child),
                            value,
                            at: at.clone().index(position),
                        });
                        continue;
                    }
                    pending.push((child, at.clone().index(position), child_walk));
                }
            }
            _ => {}
        }
    }
    sites.sort_by_key(|site| site.offset);
    row_sites.sort_by_key(|site| site.offset);
    Ok((sites, row_sites))
}

/// Run the FR-322 operand checks in their order, before any graph meaning
/// check, then rebuild every node's structural key. The first defect of the
/// first step that has one is refused as `malformed_wire`.
#[ix_trace_rs::string_edge]
pub(super) fn check(
    bytes: &[u8],
    wire: &RawCheckedPackageWireV2,
    slot_ids: &[super::CheckedNodeId],
    evidence: &CheckedPackageEvidence,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    let node_count = wire.semantic_graph.nodes.len();
    let rows = &wire.semantic_graph.operand_rows;
    let graph_at = || JsonPointer::root().key("semantic_graph");
    // 1. Rows.
    if rows.len() != node_count {
        return Err(malformed(graph_at().key("operand_rows")));
    }
    let (sites, row_sites) = collect_sites(bytes, node_count, evidence)?;
    // 2. Placement. A foreign key's segments name its dependency's graph.
    if let Some(site) = sites
        .iter()
        .find(|site| site.foreign_graph.is_none() && site.slot != site.holder.is_some())
    {
        return Err(malformed(site.at.clone()));
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
            .then_some((site.offset, &site.at))
    });
    let bad_entry = row_sites.iter().find_map(|site| {
        usize::try_from(site.value)
            .ok()
            .is_none_or(|value| value >= node_count)
            .then_some((site.offset, &site.at))
    });
    if let Some((_, at)) = [out_of_range, bad_entry]
        .into_iter()
        .flatten()
        .min_by_key(|(offset, _)| *offset)
    {
        return Err(malformed(at.clone()));
    }
    // 4. Numbering by first occurrence, node by node in graph order.
    let mut slots_by_node: Vec<Vec<&SegmentSite>> = vec![Vec::new(); node_count];
    for site in &sites {
        if let (true, Some(holder), None) = (site.slot, site.holder, site.foreign_graph) {
            slots_by_node[holder].push(site);
        }
    }
    for (node, slots) in slots_by_node.iter().enumerate() {
        let mut named = 0u64;
        for site in slots {
            if site.value > named {
                return Err(malformed(site.at.clone()));
            }
            named = named.max(site.value + 1);
        }
        let mut distinct = std::collections::BTreeSet::new();
        for (position, entry) in rows[node].iter().enumerate() {
            let unnamed = u64::try_from(position).ok().is_none_or(|p| p >= named);
            if unnamed || !distinct.insert(*entry) {
                return Err(malformed(
                    graph_at().key("operand_rows").index(node).index(position),
                ));
            }
        }
    }
    // 5. Cycle, then the keys themselves.
    rebuild_keys(
        &slot_ids.iter().collect::<Vec<_>>(),
        Some(rows),
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
    )
}

/// Each node's own ID as written, before its slots are resolved.
pub(super) fn slot_ids(wire: &RawCheckedPackageWireV2) -> Vec<super::CheckedNodeId> {
    wire.semantic_graph
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect()
}

/// Rewrite every node-held operand slot of the checked nodes to the document
/// index segment of the node its row names. The rest of the reader compares
/// operands by graph index, so a slot is read through its row once, here.
/// Every slot was range-checked by [`check`].
pub(super) fn resolve_slots(wire: &mut RawCheckedPackageWireV2) {
    let graph = &mut wire.semantic_graph;
    let rows = &graph.operand_rows;
    for (node, row) in graph.nodes.iter_mut().zip(rows) {
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
        resolve_value_slots(&mut node.body, row);
    }
}

fn resolve_value_slots(body: &mut serde_json::Value, row: &[u64]) {
    let mut pending = vec![body];
    while let Some(value) = pending.pop() {
        match value {
            serde_json::Value::Array(items) => pending.extend(items.iter_mut()),
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
                            if let Some(entry) = slot
                                .and_then(|slot| usize::try_from(slot).ok())
                                .and_then(|slot| row.get(slot))
                            {
                                *segment = serde_json::json!({ "node": entry });
                            }
                        }
                    }
                }
                pending.extend(object.values_mut());
            }
            _ => {}
        }
    }
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
    use super::contains_inline_node_path;

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
