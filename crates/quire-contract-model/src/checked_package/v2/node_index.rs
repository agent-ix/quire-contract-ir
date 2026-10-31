//! FR-322 operand-index checks before semantic-graph admission.
//!
//! The wire is walked through borrowed source spans so an index in any node
//! reference has a source pointer without building a second document tree.
//! Source spans avoid serde_json recursion at deep sibling terms.

use super::{CheckedNodePathSegment, RawCheckedPackageWireV2, ValidationFailure};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
use std::sync::Arc;

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

struct IndexSite {
    offset: usize,
    index: u64,
    graph_nodes: usize,
    at: JsonPointer,
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
                            Some(b"term") => Key::Term,
                            _ => Key::Other,
                        };
                        *keys |= match current {
                            Key::Package => 1,
                            Key::Path => 2,
                            Key::Ordinal => 4,
                            Key::Node => 8,
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
                    let is_index = members == 1
                        && keys == 8
                        && bytes
                            .get(start + b"{\"node\":".len()..at)
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

/// Check every graph-local index before any graph meaning check, then reject
/// the first own-path edge in a cycle by holder and path position.
pub(super) fn check(
    bytes: &[u8],
    wire: &RawCheckedPackageWireV2,
    evidence: &CheckedPackageEvidence,
) -> Result<Vec<Arc<super::CheckedNodeId>>, ValidationFailure> {
    let node_count = wire.semantic_graph.nodes.len();
    let mut pending = vec![(bytes, JsonPointer::root(), node_count)];
    let mut indices = Vec::new();
    while let Some((raw, at, graph_nodes)) = pending.pop() {
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
                        if object.len() != 1 || object[0].0 != "node" {
                            continue;
                        }
                        let member = object[0].1;
                        let index: u64 = serde_json::from_slice(member)
                            .map_err(|_| malformed(at.clone().key("path").index(slot)))?;
                        let offset =
                            (member.as_ptr() as usize).saturating_sub(bytes.as_ptr() as usize);
                        indices.push(IndexSite {
                            offset,
                            index,
                            graph_nodes,
                            at: at.clone().key("path").index(slot).key("node"),
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
                        .map(|package| package.graph().nodes.len())
                } else {
                    None
                };
                for (name, child) in members.into_iter().rev() {
                    let child_graph_nodes = if name == "node" {
                        if is_external_key {
                            external_graph_nodes.unwrap_or(usize::MAX)
                        } else {
                            graph_nodes
                        }
                    } else {
                        graph_nodes
                    };
                    pending.push((child, at.clone().key(&name), child_graph_nodes));
                }
            }
            Some(b'[') => {
                let items = array_items(raw).ok_or_else(|| malformed(at.clone()))?;
                for (position, child) in items.into_iter().enumerate().rev() {
                    pending.push((child, at.clone().index(position), graph_nodes));
                }
            }
            _ => {}
        }
    }
    if let Some(site) = indices
        .iter()
        .filter(|site| {
            usize::try_from(site.index)
                .ok()
                .is_none_or(|index| index >= site.graph_nodes)
        })
        .min_by_key(|site| site.offset)
    {
        return Err(malformed(site.at.clone()));
    }

    let edges: Vec<Vec<(usize, usize)>> = wire
        .semantic_graph
        .nodes
        .iter()
        .map(|node| {
            node.node_id
                .path
                .iter()
                .enumerate()
                .filter_map(|(slot, segment)| match segment {
                    CheckedNodePathSegment::Index(index) => {
                        usize::try_from(*index).ok().map(|target| (slot, target))
                    }
                    CheckedNodePathSegment::Name(_) | CheckedNodePathSegment::Node(_) => None,
                })
                .collect()
        })
        .collect();
    // Kosaraju's two heap-backed passes make the cycle decision linear in
    // nodes plus index segments, independent of graph order and chain depth.
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
                return Err(malformed(
                    JsonPointer::root()
                        .key("semantic_graph")
                        .key("nodes")
                        .index(node)
                        .key("node_id")
                        .key("path")
                        .index(slot)
                        .key("node"),
                ));
            }
        }
    }
    let mut keys: Vec<Option<Arc<super::CheckedNodeId>>> = vec![None; node_count];
    for node in order {
        let source = &wire.semantic_graph.nodes[node].node_id;
        let path = source
            .path
            .iter()
            .map(|segment| match segment {
                CheckedNodePathSegment::Name(name) => CheckedNodePathSegment::Name(name.clone()),
                CheckedNodePathSegment::Node(key) => CheckedNodePathSegment::Node(Arc::clone(key)),
                CheckedNodePathSegment::Index(index) => {
                    let target = usize::try_from(*index).expect("index was checked in bounds");
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
