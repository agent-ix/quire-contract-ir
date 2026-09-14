//! Caller-lowerable hard ceilings shared by predicate and temporal bridges.

/// Owner-enforced bridge ceilings.
pub const OWNER_MAX: BridgeLimits = BridgeLimits {
    document_bytes: 64 * 1024 * 1024,
    json_depth: 256,
    string_bytes: 1024 * 1024,
    predicates: 10_000,
    facts: 10_000,
    formula_nodes: 10_000,
    formula_depth: 256,
    positions: 10_000,
    valuations: 10_000_000,
    causes: 1_024,
    visited_work: 128 * 1024 * 1024,
    allocation_bytes: usize::MAX,
};

/// Caller-selected ceilings, always clamped to the bridge's owner maxima.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeLimits {
    /// Maximum bytes in one bridge input or output document.
    pub document_bytes: usize,
    /// Maximum JSON object/array nesting.
    pub json_depth: usize,
    /// Maximum encoded bytes in one string or member name.
    pub string_bytes: usize,
    /// Maximum checked-predicate population.
    pub predicates: usize,
    /// Maximum fact/support population.
    pub facts: usize,
    /// Maximum emitted TL formula occurrences.
    pub formula_nodes: usize,
    /// Maximum native/TL formula root-to-leaf depth.
    pub formula_depth: usize,
    /// Maximum observation positions in one projection.
    pub positions: usize,
    /// Maximum explicit position-by-proposition valuation cells.
    pub valuations: usize,
    /// Maximum decision causes.
    pub causes: usize,
    /// Maximum deterministic byte/field/node visits.
    pub visited_work: usize,
    /// Deterministic reservation failpoint; callers may lower it for tests.
    pub allocation_bytes: usize,
}

impl BridgeLimits {
    /// Returns each caller ceiling intersected with its owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            document_bytes: minimum(self.document_bytes, OWNER_MAX.document_bytes),
            json_depth: minimum(self.json_depth, OWNER_MAX.json_depth),
            string_bytes: minimum(self.string_bytes, OWNER_MAX.string_bytes),
            predicates: minimum(self.predicates, OWNER_MAX.predicates),
            facts: minimum(self.facts, OWNER_MAX.facts),
            formula_nodes: minimum(self.formula_nodes, OWNER_MAX.formula_nodes),
            formula_depth: minimum(self.formula_depth, OWNER_MAX.formula_depth),
            positions: minimum(self.positions, OWNER_MAX.positions),
            valuations: minimum(self.valuations, OWNER_MAX.valuations),
            causes: minimum(self.causes, OWNER_MAX.causes),
            visited_work: minimum(self.visited_work, OWNER_MAX.visited_work),
            allocation_bytes: self.allocation_bytes,
        }
    }
}

impl Default for BridgeLimits {
    fn default() -> Self {
        OWNER_MAX
    }
}

const fn minimum(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}
