//! FR-025 native checked-predicate to TL Boolean bridge.

mod admission;
mod artifacts;
mod decision;
mod definition;
mod reader;
mod valuation;

pub use admission::{project, TargetSelection};
pub use artifacts::{PredicateCorrespondence, PredicateProjection, ValidatedPredicateProjection};
pub use decision::{
    CompletenessGap, PredicateCause, PredicateCauseCode, PredicateCauseDimension,
    PredicateDecision, PredicateProjectionDecision, PredicateProjectionKind,
    PredicateValuationDecision, PredicateValuationKind,
};
pub use definition::{PredicateDefinition, PredicateRef};
pub use reader::{read_projection, read_valuation, ExpectedProjection, ExpectedValuation};
pub use valuation::value;

/// Selected bridge profile.
pub const PROFILE: &str = "quire.contract.native-predicate-tl-projection/v1";
/// Predicate identity profile.
pub const PREDICATE_REF_PROFILE: &str = "quire.contract.native-predicate-ref/v1";
/// Projection decision document profile.
pub const PROJECTION_DECISION_PROFILE: &str =
    "quire.contract.native-predicate-projection-decision/v1";
/// Valuation decision document profile.
pub const VALUATION_DECISION_PROFILE: &str =
    "quire.contract.native-predicate-valuation-decision/v1";
/// Projection-set identity profile.
pub const PROJECTION_REF_PROFILE: &str = "quire.contract.native-predicate-projection-set/v1";
/// Domain for exact emitted signal-catalog bytes.
pub const SIGNAL_ARTIFACT_PROFILE: &str = "quire.contract.tl-signal-catalog-artifact/v1";
/// Domain for exact emitted proposition-map bytes.
pub const MAP_ARTIFACT_PROFILE: &str = "quire.contract.tl-proposition-map-artifact/v1";
