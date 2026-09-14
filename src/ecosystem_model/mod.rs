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
    CheckedManifestSet, EcosystemLimits, ExpectedCampaign, ExpectedRepository, ManifestEdge,
    ManifestEdgeKind, ManifestGap, ManifestNode,
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
/// Lowercase SHA-256 of [`MANIFEST_SCHEMA_BYTES`].
pub const MANIFEST_SCHEMA_SHA256: &str =
    "e9fb2e4f1f6657e3e1015304bd602de670999ed91b6879ab30dea30303d757ce";
/// Canonical immutable JSON Schema for the v1 exported model document.
pub const MODEL_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/temporal-ecosystem-model-v1.schema.json");
/// Lowercase SHA-256 of [`MODEL_SCHEMA_BYTES`].
pub const MODEL_SCHEMA_SHA256: &str =
    "c41e60466661fe8197410efd71229e8081d799f7078e72d7f7292021502e471d";
