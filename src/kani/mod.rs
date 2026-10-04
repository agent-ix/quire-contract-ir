//! Shared bounded-Kani profile, ABI, outcome, and dispatch contracts.
//!
//! Semantic-family lowerings deliberately live outside this module.  This
//! boundary validates a finite offered population before any harness can add a
//! symbolic constraint, and preserves every non-success result as non-Boolean.

mod abi;
mod arithmetic;
mod collections;
mod dispatch;
mod objects;
mod outcome;
mod profile;

pub use abi::{
    FiniteInput, FiniteObject, FiniteReference, PopulationCompleteness, ResourceBounds,
    ValidatedFiniteInput,
};
pub use arithmetic::{lower_checked_arithmetic, ArithmeticLowering, CheckedArithmeticRequest};
pub use collections::{lower_query, CollectionLowering, CollectionQuery, QueryKind};
pub use dispatch::{DispatchError, DispatchIndex, ModuleDescriptor, SemanticFamily};
pub use objects::{lower_reaches, GraphLowering, GraphRequest};
pub use outcome::{
    KaniOutcome, KaniOutcomeError, KaniOutcomeKind, KaniProviderRecord, KaniProviderResult,
};
pub use profile::{
    CapabilityDisposition, CapabilityEntry, KaniProfile, ProfileError, ProfileSelection,
};

/// First selected bounded Kani profile family.
pub const PROFILE: &str = "kani-bounded/1";
