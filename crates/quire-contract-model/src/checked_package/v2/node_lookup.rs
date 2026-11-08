// SPDX-License-Identifier: AGPL-3.0-or-later
//! How a validator finds the graph position of a node id: the one lookup the
//! joins need, kept apart from the container that answers it so a validator
//! does not name that container.

use crate::checked_package::shared::CheckedNodeId;
use std::collections::BTreeMap;

/// The position of a node id in the semantic graph's node list.
pub(super) trait NodeLookup {
    fn position(&self, id: &CheckedNodeId) -> Option<&usize>;
}

impl NodeLookup for BTreeMap<&CheckedNodeId, usize> {
    fn position(&self, id: &CheckedNodeId) -> Option<&usize> {
        self.get(id)
    }
}
