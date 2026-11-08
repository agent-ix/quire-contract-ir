// SPDX-License-Identifier: AGPL-3.0-or-later
//! How a validator finds the graph position of a node id: the one lookup the
//! joins need, kept apart from the container that answers it so a validator
//! does not name that container.

use crate::checked_package::shared::CheckedNodeId;

/// The position of a node id in the semantic graph's node list.
pub(super) trait NodeLookup {
    fn position(&self, id: &CheckedNodeId) -> Option<&usize>;
}

/// Rows sorted by node id, searched by bisection.
impl NodeLookup for [(&CheckedNodeId, usize)] {
    fn position(&self, id: &CheckedNodeId) -> Option<&usize> {
        let at = self
            .binary_search_by(|(candidate, _)| (*candidate).cmp(id))
            .ok()?;
        self.get(at).map(|(_, position)| position)
    }
}

/// The graph validator's charged index uses the same node-position lookup.
impl NodeLookup for super::graph_storage::GraphIndex<'_> {
    fn position(&self, id: &CheckedNodeId) -> Option<&usize> {
        self.get(id)
    }
}
