//! FR-026 native temporal-subject to TL correspondence bridge.

mod admission;
mod correspondence;
mod decision;
mod formula;
mod join;
mod reader;
mod request;
mod valuation;

pub use admission::{
    project, ObservationViews, PositionValuations, TargetContract, TargetSelection,
};
pub use correspondence::{TemporalProjection, TemporalValuationRow, ValidatedTemporalProjection};
pub use decision::{
    JoinComparison, TemporalCause, TemporalCauseCode, TemporalCauseDimension, TemporalDecision,
    TemporalJoinDecision, TemporalJoinKind, TemporalJoinRelation, TemporalProjectionDecision,
    TemporalProjectionKind,
};
pub use join::{join, ValidatedTemporalJoin};
pub use reader::{read_join, read_projection, ExpectedTemporalJoin, ExpectedTemporalProjection};

/// Selected native/TL correspondence profile.
pub const PROFILE: &str = "quire.contract.native-temporal-correspondence/v1";
/// Projection decision wire profile.
pub const PROJECTION_DECISION_PROFILE: &str =
    "quire.contract.native-temporal-projection-decision/v1";
/// Join decision wire profile.
pub const JOIN_DECISION_PROFILE: &str = "quire.contract.native-temporal-join-decision/v1";
/// Domain for the complete correspondence tuple.
pub const CORRESPONDENCE_PROFILE: &str = "quire.contract.native-temporal-correspondence-ref/v1";
/// Domain for future trace identities constructed by this bridge.
pub const TRACE_ID_PROFILE: &str = "quire.contract.native-temporal-trace/v1";
/// Domain for past history identities constructed by this bridge.
pub const HISTORY_ID_PROFILE: &str = "quire.contract.native-temporal-history/v1";
