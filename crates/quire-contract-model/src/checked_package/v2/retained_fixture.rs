//! Fixture helpers: indexes and admission of a package built by hand.

use super::*;

/// The node index of a fixture package, charged to no one.
pub(super) fn retained_node_index(wire: &CheckedPackageWireV2) -> PositionIndex {
    retained_index::node_index(wire, &mut NativeJoinBudget::unlimited()).expect("unlimited budget")
}

/// The source index of a fixture package, charged to no one.
pub(super) fn retained_source_index(wire: &CheckedPackageWireV2) -> PositionIndex {
    crate::checked_package::selected_population::source_index(&wire.source_map, &mut NativeJoinBudget::unlimited())
        .expect("unlimited budget")
}

impl CheckedPackageV2 {
    pub(super) fn retain_original_admission(self) -> Self {
        self.retain_admission(&mut NativeJoinBudget::unlimited())
            .expect("unlimited budget")
    }

    /// Rebuilds the node and source indexes after a fixture changes the graph.
    pub(in crate::checked_package) fn reindex(&mut self) {
        self.node_index = Arc::new(retained_node_index(&self.wire));
        self.source_index = Arc::new(retained_source_index(&self.wire));
    }
}
