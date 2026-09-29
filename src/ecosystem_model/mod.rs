//! FR-027 bounded, descriptive temporal-ecosystem model.

mod decision;
mod document;
mod graph;
pub mod manifest;
mod reader;

pub use decision::{ModelCauseCode, ModelDecision};
pub use document::{export, ImprovementProposal, ModelCounts, ModelDocument};
pub use graph::{ModelAdjacency, ModelAdjacentEdge};
pub use manifest::{
    CheckedManifestSet, EcosystemLimits, ExpectedCampaign, ManifestEdge, ManifestEdgeKind,
    ManifestGap, ManifestNode,
};
pub use reader::{read, ValidatedEcosystemModel};

/// Immutable input-manifest contract profile.
pub const MANIFEST_PROFILE: &str = "quire.contract.temporal-ecosystem-manifest/v1";
/// Immutable output-model contract profile.
pub const MODEL_PROFILE: &str = "quire.contract.temporal-ecosystem-model/v1";
/// Domain-separated manifest content identity.
pub const MANIFEST_ID_PROFILE: &str = "quire.contract.temporal-ecosystem-manifest-ref/v1";
/// Domain-separated improvement-proposal identity.
pub const PROPOSAL_ID_PROFILE: &str = "quire.contract.temporal-ecosystem-proposal/v1";

/// Canonical immutable JSON Schema for the v1 manifest document.
pub const MANIFEST_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/temporal-ecosystem-manifest-v1.schema.json");
/// Canonical immutable JSON Schema for the v1 exported model document.
pub const MODEL_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/temporal-ecosystem-model-v1.schema.json");
