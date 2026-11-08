//! An exact index of an admitted package's dependency closure.

use super::read_charge::{self, Site};
use super::{CheckedPackageV2, NativeJoinBudget, PackageReadStage};
use crate::checked_package::common::ValidationFailure;
use std::sync::Arc;

/// The packages reachable through a package's direct dependencies, one per
/// identity, sorted by package name and searched by binary search.
#[derive(Clone, Default)]
pub(super) struct ClosureIndex {
    /// `None` when the closure is empty, which owns no heap.
    packages: Option<Arc<[Arc<CheckedPackageV2>]>>,
}

/// The bytes of the block `Arc<[T]>` requests for `count` items: the strong
/// and weak counts and the items.
fn arc_slice_bytes<T>(count: usize) -> u128 {
    let align = std::mem::align_of::<T>().max(std::mem::align_of::<usize>());
    let bytes = 2 * std::mem::size_of::<usize>() + count.saturating_mul(std::mem::size_of::<T>());
    bytes.next_multiple_of(align) as u128
}

impl ClosureIndex {
    pub(super) fn get(&self, identity: &str) -> Option<&Arc<CheckedPackageV2>> {
        let packages = self.packages.as_deref()?;
        packages
            .binary_search_by(|package| package.name().cmp(identity))
            .ok()
            .map(|position| &packages[position])
    }

    fn len(&self) -> usize {
        self.packages.as_deref().map_or(0, <[_]>::len)
    }

    fn iter(&self) -> impl Iterator<Item = &Arc<CheckedPackageV2>> {
        self.packages.as_deref().unwrap_or_default().iter()
    }

    /// Builds the index after exact closure conflict validation: every direct
    /// package and each package its own closure holds, once per identity. The
    /// candidate list and the index are charged to `budget` before they are
    /// allocated.
    pub(super) fn from_direct(
        packages: &[Arc<CheckedPackageV2>],
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ValidationFailure> {
        if packages.is_empty() {
            return Ok(Self::default());
        }
        let site = Site::new(PackageReadStage::Retained, 0, "closure_index");
        let candidates = packages.iter().fold(0usize, |sum, package| {
            sum.saturating_add(1)
                .saturating_add(package.closure_index.len())
        });
        let mut unique: Vec<&Arc<CheckedPackageV2>> =
            read_charge::with_capacity(budget, site, candidates)?;
        for package in packages {
            unique.push(package);
            unique.extend(package.closure_index.iter());
        }
        unique.sort_unstable_by(|left, right| left.name().cmp(right.name()));
        unique.dedup_by(|left, right| left.name() == right.name());
        read_charge::charge(
            budget,
            site,
            arc_slice_bytes::<Arc<CheckedPackageV2>>(unique.len()),
        )?;
        let packages: Arc<[Arc<CheckedPackageV2>]> = unique.iter().copied().cloned().collect();
        Ok(Self {
            packages: Some(packages),
        })
    }
}
