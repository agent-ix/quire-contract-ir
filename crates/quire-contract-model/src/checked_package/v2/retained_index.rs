//! Position indexes an admitted package retains: sorted exact-capacity slices
//! of row positions searched by binary search, each charged to the read's
//! budget before it is allocated.

use super::read_charge::{self, Site};
use super::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageWireV2, CheckedSemanticNodeV2,
    CheckedSourceMapEntry, NativeJoinBudget, PackageReadStage,
};
use crate::checked_package::common::ValidationFailure;
use std::cmp::Ordering;

/// Row positions sorted by the key of the row each names. The rows are not
/// owned: a lookup passes the table the positions index.
#[derive(Clone, Debug, Default)]
pub(in crate::checked_package) struct PositionIndex(Box<[usize]>);

impl PositionIndex {
    /// The positions `0..count` sorted by `order`, with the position as the
    /// final tie-break so the first of equal keys is found first. The block is
    /// charged at `site` before it is requested.
    fn sorted(
        budget: &mut NativeJoinBudget,
        site: Site,
        count: usize,
        mut order: impl FnMut(usize, usize) -> Ordering,
    ) -> Result<Self, ValidationFailure> {
        let mut positions: Vec<usize> = read_charge::with_capacity(budget, site, count)?;
        positions.extend(0..count);
        positions
            .sort_unstable_by(|left, right| order(*left, *right).then_with(|| left.cmp(right)));
        Ok(Self(positions.into_boxed_slice()))
    }

    /// The first position whose row key equals the probe, where `compare`
    /// orders a position's row key against the probe.
    fn find(&self, compare: impl Fn(usize) -> Ordering) -> Option<usize> {
        let first = self
            .0
            .partition_point(|&position| compare(position) == Ordering::Less);
        self.0
            .get(first)
            .copied()
            .filter(|&position| compare(position) == Ordering::Equal)
    }

    /// The position of the node with `id` among `nodes`.
    pub(in crate::checked_package) fn node<Body>(
        &self,
        nodes: &[CheckedSemanticNodeV2<Body>],
        id: &CheckedNodeId,
    ) -> Option<usize> {
        self.find(|position| {
            nodes
                .get(position)
                .map_or(Ordering::Greater, |node| node.node_id.cmp(id))
        })
    }

    /// Read a borrowed foreign-coordinate key against the retained document IDs.
    pub(in crate::checked_package) fn node_probe<Body>(
        &self,
        nodes: &[CheckedSemanticNodeV2<Body>],
        probe: &dyn crate::checked_package::shared::NodeKeyProbe,
    ) -> Option<usize> {
        self.find(|position| {
            nodes.get(position).map_or(Ordering::Greater, |node| {
                (&node.node_id as &dyn crate::checked_package::shared::NodeKeyProbe).cmp(probe)
            })
        })
    }

    /// This index without the row at `position`, as a damaged index.
    #[cfg(test)]
    pub(in crate::checked_package) fn without(&self, position: usize) -> Self {
        Self(
            self.0
                .iter()
                .copied()
                .filter(|&at| at != position)
                .collect(),
        )
    }

    /// The identities of `nodes` in ascending order.
    pub(in crate::checked_package) fn node_ids<'n, Body>(
        &'n self,
        nodes: &'n [CheckedSemanticNodeV2<Body>],
    ) -> impl Iterator<Item = &'n CheckedNodeId> {
        self.0.iter().map(|&position| &nodes[position].node_id)
    }

    /// The position of the source-map entry for an occurrence among `entries`.
    pub(in crate::checked_package) fn source(
        &self,
        entries: &[CheckedSourceMapEntry],
        id: &CheckedNodeId,
        role: &CheckedOccurrenceRole,
        ordinal: u64,
    ) -> Option<usize> {
        self.find(|position| {
            entries.get(position).map_or(Ordering::Greater, |entry| {
                entry
                    .node_id
                    .cmp(id)
                    .then_with(|| entry.role.cmp(role))
                    .then_with(|| entry.ordinal.cmp(&ordinal))
            })
        })
    }
}

/// A lookup of a graph node's position by its identity: the retained sorted
/// index, or a map a validation builds over a graph it has not retained.
pub(in crate::checked_package) trait NodePositions {
    /// The position of the node `id`, where `id_at` names the node at a
    /// position.
    fn position<'n>(
        &self,
        id: &CheckedNodeId,
        id_at: &dyn Fn(usize) -> &'n CheckedNodeId,
    ) -> Option<usize>;
}

impl NodePositions for PositionIndex {
    fn position<'n>(
        &self,
        id: &CheckedNodeId,
        id_at: &dyn Fn(usize) -> &'n CheckedNodeId,
    ) -> Option<usize> {
        self.find(|position| id_at(position).cmp(id))
    }
}

impl NodePositions for std::collections::BTreeMap<CheckedNodeId, usize> {
    fn position<'n>(
        &self,
        id: &CheckedNodeId,
        _id_at: &dyn Fn(usize) -> &'n CheckedNodeId,
    ) -> Option<usize> {
        self.get(id).copied()
    }
}

/// The node index of a package's graph, charged at `Retained` /
/// `"node_index"`.
pub(in crate::checked_package) fn node_index(
    wire: &CheckedPackageWireV2,
    budget: &mut NativeJoinBudget,
) -> Result<PositionIndex, ValidationFailure> {
    let nodes = &wire.semantic_graph.nodes;
    let site = Site::new(PackageReadStage::Retained, 0, "node_index");
    PositionIndex::sorted(budget, site, nodes.len(), |left, right| {
        nodes[left].node_id.cmp(&nodes[right].node_id)
    })
}

/// The source index of a package's source map, charged at `Retained` /
/// `"source_index"`.
pub(in crate::checked_package) fn source_index(
    entries: &[CheckedSourceMapEntry],
    budget: &mut NativeJoinBudget,
) -> Result<PositionIndex, ValidationFailure> {
    let site = Site::new(PackageReadStage::Retained, 0, "source_index");
    PositionIndex::sorted(budget, site, entries.len(), |left, right| {
        let (left, right) = (&entries[left], &entries[right]);
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.role.cmp(&right.role))
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    })
}
