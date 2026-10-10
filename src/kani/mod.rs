//! Shared bounded-Kani profile, ABI, outcome, and dispatch contracts.
//!
//! Semantic-family lowerings deliberately live outside this module.  This
//! boundary validates a finite offered population before any harness can add a
//! symbolic constraint, and preserves every non-success result as non-Boolean.
//!
//! FR-039-AC-3, TC-055: QSL owns the terminal result and record, so neither
//! provider type is available through this module:
//!
//! ```compile_fail,E0432
//! use quire_contract_ir::kani::KaniProviderResult;
//! ```
//!
//! ```compile_fail,E0432
//! use quire_contract_ir::kani::KaniProviderRecord;
//! ```

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
use outcome::NonSuccessKind;
pub use outcome::{KaniOutcome, KaniOutcomeError, KaniOutcomeKind};
pub use profile::{
    CapabilityDisposition, CapabilityEntry, KaniProfile, ProfileError, ProfileSelection,
};

/// First selected bounded Kani profile family.
pub const PROFILE: &str = "kani-bounded/1";
