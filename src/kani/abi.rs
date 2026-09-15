//! Finite input firewall shared by all bounded-Kani semantic modules.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{KaniOutcome, KaniOutcomeKind, ProfileSelection};

/// Explicit population completeness state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopulationCompleteness {
    /// Every member of the selected finite universe is represented.
    Complete,
    /// A required member or observation is unavailable.
    Incomplete,
}

/// Exact resource ceilings selected before validation or harness construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceBounds {
    /// Maximum objects in the complete universe.
    pub max_objects: usize,
    /// Maximum reference edges in the complete universe.
    pub max_references: usize,
    /// Maximum aggregate input bytes supplied by the owner adapter.
    pub max_input_bytes: usize,
}

/// One nominal identity-bearing finite object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FiniteObject {
    /// Exact object identity in one selected universe.
    pub identity: String,
    /// Exact nominal object type.
    pub type_id: String,
    /// Snapshot which owns this object value.
    pub snapshot_id: String,
}

/// One exact finite reference edge.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FiniteReference {
    /// Identity of the referring object.
    pub source_id: String,
    /// Declared reference field identity.
    pub field_id: String,
    /// Identity of the target object.
    pub target_id: String,
}

/// Offered input before its finite population has been validated.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FiniteInput {
    /// Exact source model identity.
    pub model_id: String,
    /// Exact source/snapshot/invocation identity selected by the caller.
    pub source_id: String,
    /// Shared profile selection whose bounds interpret this input.
    pub profile: ProfileSelection,
    /// Completeness claim for this offered population.
    pub completeness: PopulationCompleteness,
    /// Explicit resource ceilings.
    pub bounds: ResourceBounds,
    /// Exact aggregate input byte count retained by the adapter.
    pub input_bytes: usize,
    /// Closed nominal object universe.
    pub objects: Vec<FiniteObject>,
    /// Exact reference edges over that universe.
    pub references: Vec<FiniteReference>,
}

/// Private proof that an offered input crossed the validation firewall.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedFiniteInput(FiniteInput);

impl ValidatedFiniteInput {
    /// Returns the exact validated input; no partial representation is exposed on failure.
    #[must_use]
    pub const fn input(&self) -> &FiniteInput {
        &self.0
    }
}

impl FiniteInput {
    /// Validates a claimed complete finite universe before any Kani assumption is permitted.
    pub fn validate(self) -> Result<ValidatedFiniteInput, KaniOutcome> {
        let context = self.profile.revision.clone();
        if self.model_id.trim().is_empty() || self.source_id.trim().is_empty() {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_identity_invalid",
                self.source_id,
                context,
            ));
        }
        if self.completeness == PopulationCompleteness::Incomplete {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::IncompleteInput,
                "kani_population_incomplete",
                self.source_id,
                context,
            ));
        }
        if self.bounds.max_objects == 0 || self.bounds.max_input_bytes == 0 {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_bound_invalid",
                self.source_id,
                context,
            ));
        }
        if self.input_bytes > self.bounds.max_input_bytes
            || self.objects.len() > self.bounds.max_objects
            || self.references.len() > self.bounds.max_references
        {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::ResourceExhausted,
                "kani_bound_exhausted",
                self.source_id,
                context,
            ));
        }
        let mut objects = BTreeMap::new();
        for object in &self.objects {
            if object.identity.trim().is_empty()
                || object.type_id.trim().is_empty()
                || object.snapshot_id.trim().is_empty()
                || objects.insert(object.identity.as_str(), object).is_some()
            {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::InvalidInput,
                    "kani_population_invalid",
                    self.source_id,
                    context,
                ));
            }
        }
        let mut references = BTreeSet::new();
        for reference in &self.references {
            if reference.field_id.trim().is_empty()
                || !objects.contains_key(reference.source_id.as_str())
                || !objects.contains_key(reference.target_id.as_str())
                || !references.insert((
                    reference.source_id.as_str(),
                    reference.field_id.as_str(),
                    reference.target_id.as_str(),
                ))
            {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::InvalidInput,
                    "kani_reference_invalid",
                    self.source_id,
                    context,
                ));
            }
        }
        Ok(ValidatedFiniteInput(self))
    }
}
