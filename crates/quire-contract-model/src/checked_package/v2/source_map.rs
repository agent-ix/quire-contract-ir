// SPDX-License-Identifier: AGPL-3.0-or-later
//! The source-map check: the map is exactly the graph's occurrence set, over
//! locked, current, non-overlapping half-open regions.
//!
//! Implements: FR-038 "Budget-charged strict read" (stage `SourceMap`).
//!
//! Every table is a sorted exact-capacity slice charged to the caller's budget
//! at `PackageRead(SourceMap, entry)` before it is requested.

use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::WorkMeter;
use crate::checked_package::common::{exceeds, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedSourceMapEntry, CheckedSourceRef,
};

/// One occurrence key: node, role and ordinal.
type Key<'a> = (&'a CheckedNodeId, &'a CheckedOccurrenceRole, u64);

/// Requires the source map to be exactly the graph's occurrence set, with
/// nonempty, locked, current, non-overlapping half-open regions.
pub(in crate::checked_package) fn validate_source_map_entries<'a, N>(
    nodes: N,
    source_map: &[CheckedSourceMapEntry],
    locked_sources: &[CheckedSourceRef],
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure>
where
    N: Iterator<Item = (&'a CheckedNodeId, &'a [CheckedOccurrence])> + Clone,
{
    let root = Path::root().key("source_map");
    let site = |index: usize, at: Path<'static>| {
        Site::new(PackageReadStage::SourceMap, index, "source_map", at)
    };
    let invalid = CheckedPackageRefusalCode::InvalidSourceMap;
    let occurrences = nodes
        .clone()
        .fold(0_usize, |total, (_, held)| total.saturating_add(held.len()));
    // Every occurrence the graph holds, ascending and without repeats.
    let mut expected: Vec<Key<'_>> = stage_storage::vec(meter, &site(0, root), occurrences)?;
    expected.extend(nodes.flat_map(|(node, held)| {
        held.iter()
            .map(move |occurrence| (node, &occurrence.role, occurrence.ordinal))
    }));
    expected.sort_unstable();
    expected.dedup();
    // Every entry's key and position, ascending.
    let mut actual: Vec<(Key<'_>, usize)> =
        stage_storage::vec(meter, &site(0, root), source_map.len())?;
    actual.extend(
        source_map
            .iter()
            .enumerate()
            .map(|(position, entry)| ((&entry.node_id, &entry.role, entry.ordinal), position)),
    );
    actual.sort_unstable();
    // The first entry that repeats an earlier entry's key.
    let first_repeat = actual
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min();
    let mut locked: Vec<&CheckedSourceRef> =
        stage_storage::vec(meter, &site(0, root), locked_sources.len())?;
    locked.extend(locked_sources.iter());
    locked.sort_unstable();
    let mut count = 0_u64;
    for (entry_index, entry) in source_map.iter().enumerate() {
        let entry_at = root.index(entry_index);
        let regions_at = entry_at.key("regions");
        count = count.saturating_add(1);
        if exceeds(count, limits.occurrences) {
            return Err(stage_storage::incomplete(
                meter,
                &site(entry_index, entry_at),
                CheckedPackageLimit::Occurrences,
                limits.occurrences,
                count,
                entry_at,
            ));
        }
        if entry.regions.is_empty() {
            return Err(stage_storage::refuse(
                meter,
                &site(entry_index, regions_at),
                Refusal::new(invalid, regions_at),
            ));
        }
        if first_repeat == Some(entry_index) {
            return Err(stage_storage::refuse(
                meter,
                &site(entry_index, entry_at),
                Refusal::new(invalid, entry_at),
            ));
        }
        let before_regions = count;
        count = count.saturating_add(u64::try_from(entry.regions.len()).unwrap_or(u64::MAX));
        if exceeds(count, limits.occurrences) {
            // `before_regions` was admitted, so the first region whose
            // charge failed is the one that took the counter one past the
            // limit.
            let first_over = limits.occurrences.saturating_sub(before_regions);
            let first_over = usize::try_from(first_over).unwrap_or(usize::MAX);
            return Err(stage_storage::incomplete(
                meter,
                &site(entry_index, regions_at),
                CheckedPackageLimit::Occurrences,
                limits.occurrences,
                count,
                regions_at.index(first_over),
            ));
        }
        // The entry's regions by source, then position.
        let mut ranges: Vec<(&CheckedSourceRef, u64, u64, usize)> =
            stage_storage::vec(meter, &site(entry_index, entry_at), entry.regions.len())?;
        for (region_index, region) in entry.regions.iter().enumerate() {
            let region_at = regions_at.index(region_index);
            if locked.binary_search(&&region.source).is_err() {
                return Err(stage_storage::refuse(
                    meter,
                    &site(entry_index, region_at),
                    Refusal::new(invalid, region_at.key("source")),
                ));
            }
            if region.start >= region.end {
                return Err(stage_storage::refuse(
                    meter,
                    &site(entry_index, region_at),
                    Refusal::new(invalid, region_at),
                ));
            }
            ranges.push((&region.source, region.start, region.end, region_index));
        }
        ranges.sort_unstable();
        if let Some(pair) = ranges
            .windows(2)
            .find(|pair| pair[0].0 == pair[1].0 && pair[0].2 > pair[1].1)
        {
            let region_at = regions_at.index(pair[1].3);
            return Err(stage_storage::refuse(
                meter,
                &site(entry_index, region_at),
                Refusal::new(invalid, region_at),
            ));
        }
    }
    let agrees = actual.len() == expected.len()
        && actual
            .iter()
            .zip(&expected)
            .all(|((key, _), wanted)| key == wanted);
    if !agrees {
        // An entry naming no graph occurrence is the value at fault; an
        // occurrence no entry maps is a defect of the array as a whole.
        let stray = source_map.iter().position(|entry| {
            expected
                .binary_search(&(&entry.node_id, &entry.role, entry.ordinal))
                .is_err()
        });
        let at = stray.map_or(root, |entry| root.index(entry));
        return Err(stage_storage::refuse(
            meter,
            &site(stray.unwrap_or(0), at),
            Refusal::new(invalid, at),
        ));
    }
    Ok(())
}
