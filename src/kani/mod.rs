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

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::lower_checked_arithmetic;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::lower_query;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::lower_reaches;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::ArithmeticLowering;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::CheckedArithmeticRequest;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::CollectionLowering;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::CollectionQuery;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::QueryKind;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::GraphLowering;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::GraphRequest;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::CounterexamplePacket;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::PacketIdentity;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::ReplaySource;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::ReplayAgreement;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::WitnessReplayAgreement;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::InputReplayAgreement;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::replay_counterexample;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::Witness;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::WitnessBinding;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::WitnessCheck;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::WitnessValue;
//! ```

//! ```compile_fail,E0432
//! use quire_contract_ir::kani::WitnessValueType;
//! ```

mod abi;
mod dispatch;
mod outcome;
mod profile;

pub use abi::{
    FiniteInput, FiniteObject, FiniteReference, PopulationCompleteness, ResourceBounds,
    ValidatedFiniteInput,
};
pub use dispatch::{DispatchError, DispatchIndex, ModuleDescriptor, SemanticFamily};
use outcome::NonSuccessKind;
pub use outcome::{KaniCancellationSource, KaniOutcome, KaniOutcomeError, KaniOutcomeKind};
pub use profile::{
    CapabilityDisposition, CapabilityEntry, KaniProfile, ProfileError, ProfileSelection,
};

/// First selected bounded Kani profile family.
pub const PROFILE: &str = "kani-bounded/1";
