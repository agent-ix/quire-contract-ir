//! Shared bounded-Kani profile, ABI, outcome, provenance, and dispatch contracts.
//!
//! Semantic-family lowerings deliberately live outside this module.  This
//! boundary validates a finite offered population before any harness can add a
//! symbolic constraint, and preserves every non-success result as non-Boolean.

mod abi;
mod dispatch;
mod outcome;
mod profile;
mod provenance;

pub use abi::{
    FiniteInput, FiniteObject, FiniteReference, PopulationCompleteness, ResourceBounds,
    ValidatedFiniteInput,
};
pub use dispatch::{DispatchError, DispatchIndex, ModuleDescriptor, SemanticFamily};
pub use outcome::{KaniOutcome, KaniOutcomeKind};
pub use profile::{
    CapabilityDisposition, CapabilityEntry, KaniProfile, ProfileError, ProfileSelection,
};
pub use provenance::{ArtifactIdentity, GeneratorProvenance, ProvenanceError};

/// First selected bounded Kani profile family.
pub const PROFILE: &str = "kani-bounded/1";
