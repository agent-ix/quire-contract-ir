//! An exact, structurally shared identity index for admitted dependency closures.

use super::{CheckedDependencySelection, CheckedPackageV2};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Default)]
struct Node {
    package: Option<Arc<CheckedPackageV2>>,
    children: BTreeMap<u8, Arc<Self>>,
}

impl Drop for Node {
    fn drop(&mut self) {
        let mut pending: Vec<_> = std::mem::take(&mut self.children).into_values().collect();
        while let Some(child) = pending.pop() {
            if let Ok(mut unique) = Arc::try_unwrap(child) {
                pending.extend(std::mem::take(&mut unique.children).into_values());
            }
        }
    }
}

/// A persistent byte trie. Extending a one-edge chain copies only the new
/// identity's path; ancestor packages share every other node.
#[derive(Clone, Default)]
pub(super) struct ClosureIndex {
    root: Arc<Node>,
}

impl ClosureIndex {
    pub(super) fn get(&self, identity: &str) -> Option<&Arc<CheckedPackageV2>> {
        let mut node = self.root.as_ref();
        for byte in identity.bytes() {
            node = node.children.get(&byte)?.as_ref();
        }
        node.package.as_ref()
    }

    fn insert(&mut self, identity: &str, package: &Arc<CheckedPackageV2>) {
        let mut ancestors = Vec::with_capacity(identity.len());
        let mut node = Some(self.root.as_ref());
        for byte in identity.bytes() {
            ancestors.push((node, byte));
            node = node.and_then(|parent| parent.children.get(&byte).map(AsRef::as_ref));
        }
        let mut replacement = Arc::new(Node {
            package: Some(Arc::clone(package)),
            children: node.map(|leaf| leaf.children.clone()).unwrap_or_default(),
        });
        for (parent, byte) in ancestors.into_iter().rev() {
            let mut copy = parent.cloned().unwrap_or_default();
            copy.children.insert(byte, replacement);
            replacement = Arc::new(copy);
        }
        self.root = replacement;
    }

    /// Builds the index after exact closure conflict validation. The common
    /// one-direct-dependency case shares its child's index and adds one key.
    pub(super) fn from_direct<'a>(
        selections: &'a [CheckedDependencySelection],
        packages: &'a [Arc<CheckedPackageV2>],
    ) -> Self {
        if let (Some(selection), [package]) = (selections.first(), packages) {
            let mut index = package.closure_index.clone();
            index.insert(&selection.identity, package);
            return index;
        }
        let mut index = Self::default();
        let mut seen = BTreeSet::new();
        let mut pending: Vec<_> = selections.iter().zip(packages).collect();
        while let Some((selection, package)) = pending.pop() {
            if !seen.insert(selection.identity.as_ref()) {
                continue;
            }
            index.insert(&selection.identity, package);
            pending.extend(
                package
                    .lock()
                    .dependency_selections
                    .iter()
                    .zip(&package.direct_dependencies)
                    .map(|(entry, child)| (entry, child)),
            );
        }
        index
    }
}
