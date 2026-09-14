//! Deterministic model export and non-authoritative proposal values.

use serde::{Deserialize, Serialize};

use crate::bridge::{canonical, BridgeDigest};

use super::decision::{ModelCauseCode as Code, ModelDecision};
use super::graph::ModelAdjacency;
use super::manifest::{
    bridge_limits, resource, u64_len, CheckedManifestSet, EcosystemLimits, ManifestEdge,
    ManifestGap, ManifestNode,
};
use super::{MODEL_PROFILE, MODEL_SCHEMA_SHA256, PROPOSAL_ID_PROFILE};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModelWire {
    pub profile: String,
    pub schema_digest: String,
    pub identity: BridgeDigest,
    pub manifest_identity: BridgeDigest,
    pub campaign: String,
    pub limits: EcosystemLimits,
    pub counts: ModelCounts,
    pub nodes: Vec<ManifestNode>,
    pub edges: Vec<ManifestEdge>,
    pub gaps: Vec<ManifestGap>,
    pub adjacency: Vec<ModelAdjacency>,
    pub topological_order: Vec<String>,
}

#[derive(Serialize)]
struct ModelBody<'a> {
    profile: &'static str,
    schema_digest: &'static str,
    manifest_identity: BridgeDigest,
    campaign: &'a str,
    limits: EcosystemLimits,
    counts: ModelCounts,
    nodes: &'a [ManifestNode],
    edges: &'a [ManifestEdge],
    gaps: &'a [ManifestGap],
    adjacency: &'a [ModelAdjacency],
    topological_order: &'a [String],
}

/// Exact retained population counts by closed node and relation kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCounts {
    /// Retained repository nodes.
    pub repositories: u64,
    /// Retained component nodes.
    pub components: u64,
    /// Retained semantic-object nodes.
    pub objects: u64,
    /// Retained executable-interface nodes.
    pub interfaces: u64,
    /// Retained exact-contract-selection nodes.
    pub contracts: u64,
    /// Retained requirement nodes.
    pub requirements: u64,
    /// Retained test nodes.
    pub tests: u64,
    /// Retained review nodes.
    pub reviews: u64,
    /// Retained unresolved gaps.
    pub gaps: u64,
    /// Retained typed edges.
    pub edges: u64,
}

/// Exact canonical exported model bytes and identities.
#[derive(Clone, Debug)]
pub struct ModelDocument {
    pub(crate) wire: ModelWire,
    bytes: Box<[u8]>,
    byte_digest: BridgeDigest,
}

impl ModelDocument {
    /// Returns exact canonical model bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the model content identity computed without its identity field.
    #[must_use]
    pub const fn identity(&self) -> BridgeDigest {
        self.wire.identity
    }

    /// Returns SHA-256 of the complete enclosing bytes.
    #[must_use]
    pub const fn byte_digest(&self) -> BridgeDigest {
        self.byte_digest
    }

    /// Returns the selected manifest identity.
    #[must_use]
    pub const fn manifest_identity(&self) -> BridgeDigest {
        self.wire.manifest_identity
    }

    /// Returns deterministic outgoing adjacency for every model node.
    #[must_use]
    pub fn adjacency(&self) -> &[ModelAdjacency] {
        &self.wire.adjacency
    }

    /// Returns prerequisite-first ownership/runtime topological order.
    #[must_use]
    pub fn topological_order(&self) -> &[String] {
        &self.wire.topological_order
    }

    /// Returns the exact retained population counts.
    #[must_use]
    pub const fn counts(&self) -> ModelCounts {
        self.wire.counts
    }
}

/// Descriptive improvement request bound to one source model; no acceptance state exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImprovementProposal {
    source_model_identity: BridgeDigest,
    identity: BridgeDigest,
    description: Box<str>,
}

impl ImprovementProposal {
    /// Creates a descriptive proposal without granting authority or acceptance.
    pub fn new(
        source: &crate::ecosystem_model::ValidatedEcosystemModel,
        description: impl Into<Box<str>>,
        limits: EcosystemLimits,
    ) -> Result<Self, ModelDecision> {
        let limits = limits.effective();
        let description = description.into();
        super::manifest::validate_text(&description, limits, "proposal.description")?;
        let mut preimage = Vec::new();
        let capacity = source
            .identity()
            .as_bytes()
            .len()
            .saturating_add(description.len());
        if u64_len(capacity)? > limits.allocation_bytes {
            return Err(resource(
                "proposal",
                "proposal reservation ceiling exceeded",
            ));
        }
        preimage
            .try_reserve_exact(capacity)
            .map_err(|_| resource("proposal", "proposal reservation failed"))?;
        preimage.extend_from_slice(source.identity().as_bytes());
        preimage.extend_from_slice(description.as_bytes());
        Ok(Self {
            source_model_identity: source.identity(),
            identity: BridgeDigest::domain(PROPOSAL_ID_PROFILE, &preimage),
            description,
        })
    }

    /// Returns the immutable source model identity.
    #[must_use]
    pub const fn source_model_identity(&self) -> BridgeDigest {
        self.source_model_identity
    }

    /// Returns the descriptive proposal identity.
    #[must_use]
    pub const fn identity(&self) -> BridgeDigest {
        self.identity
    }

    /// Returns the proposed change text.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Exports the complete deterministic model from an admitted manifest.
pub fn export(
    manifest: &CheckedManifestSet,
    limits: EcosystemLimits,
) -> Result<ModelDocument, ModelDecision> {
    let limits = limits.effective();
    let counts = counts(manifest)?;
    validate_counts(counts, limits)?;
    let body = ModelBody {
        profile: MODEL_PROFILE,
        schema_digest: MODEL_SCHEMA_SHA256,
        manifest_identity: manifest.identity,
        campaign: &manifest.wire.campaign,
        limits,
        counts,
        nodes: &manifest.wire.nodes,
        edges: &manifest.wire.edges,
        gaps: &manifest.wire.gaps,
        adjacency: &manifest.graph.adjacency,
        topological_order: &manifest.graph.topological_order,
    };
    let body_bytes = canonical::encode(&body, bridge_limits(limits, false))
        .map_err(|error| resource("model", error.message()))?;
    let identity = model_identity(&body_bytes, limits)?;
    let wire = ModelWire {
        profile: MODEL_PROFILE.to_owned(),
        schema_digest: MODEL_SCHEMA_SHA256.to_owned(),
        identity,
        manifest_identity: manifest.identity,
        campaign: manifest.wire.campaign.clone(),
        limits,
        counts: body.counts,
        nodes: manifest.wire.nodes.clone(),
        edges: manifest.wire.edges.clone(),
        gaps: manifest.wire.gaps.clone(),
        adjacency: manifest.graph.adjacency.clone(),
        topological_order: manifest.graph.topological_order.clone(),
    };
    let bytes = canonical::encode(&wire, bridge_limits(limits, false))
        .map_err(|error| resource("model", error.message()))?;
    if u64_len(bytes.len())? > limits.model_bytes || u64_len(bytes.len())? > limits.allocation_bytes
    {
        return Err(resource("model", "model byte ceiling exceeded"));
    }
    let byte_digest = BridgeDigest::raw(&bytes);
    Ok(ModelDocument {
        wire,
        bytes: bytes.into_boxed_slice(),
        byte_digest,
    })
}

pub(crate) fn model_identity(
    body_bytes: &[u8],
    limits: EcosystemLimits,
) -> Result<BridgeDigest, ModelDecision> {
    let capacity = MODEL_PROFILE
        .len()
        .saturating_add(1)
        .saturating_add(body_bytes.len());
    if u64_len(capacity)? > limits.allocation_bytes {
        return Err(resource(
            "model.identity",
            "identity preimage reservation ceiling exceeded",
        ));
    }
    let mut preimage = Vec::new();
    preimage
        .try_reserve_exact(capacity)
        .map_err(|_| resource("model.identity", "identity preimage reservation failed"))?;
    preimage.extend_from_slice(MODEL_PROFILE.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(body_bytes);
    Ok(BridgeDigest::raw(&preimage))
}

pub(crate) fn validate_wire_identity(
    wire: &ModelWire,
    limits: EcosystemLimits,
) -> Result<(), ModelDecision> {
    if wire.profile != MODEL_PROFILE || wire.schema_digest != MODEL_SCHEMA_SHA256 {
        return Err(ModelDecision::new(
            Code::ContractMismatch,
            "model.contract",
            "model profile or schema digest differs",
        ));
    }
    let body = ModelBody {
        profile: MODEL_PROFILE,
        schema_digest: MODEL_SCHEMA_SHA256,
        manifest_identity: wire.manifest_identity,
        campaign: &wire.campaign,
        limits: wire.limits,
        counts: wire.counts,
        nodes: &wire.nodes,
        edges: &wire.edges,
        gaps: &wire.gaps,
        adjacency: &wire.adjacency,
        topological_order: &wire.topological_order,
    };
    let bytes = canonical::encode(&body, bridge_limits(limits, false))
        .map_err(|error| resource("model.identity", error.message()))?;
    if model_identity(&bytes, limits)? != wire.identity {
        return Err(ModelDecision::new(
            Code::IdentityMismatch,
            "model.identity",
            "model content identity differs",
        ));
    }
    Ok(())
}

fn counts(manifest: &CheckedManifestSet) -> Result<ModelCounts, ModelDecision> {
    let mut result = ModelCounts {
        repositories: 0,
        components: 0,
        objects: 0,
        interfaces: 0,
        contracts: 0,
        requirements: 0,
        tests: 0,
        reviews: 0,
        gaps: u64_len(manifest.wire.gaps.len())?,
        edges: u64_len(manifest.wire.edges.len())?,
    };
    for node in &manifest.wire.nodes {
        let count = match node {
            ManifestNode::Repository { .. } => &mut result.repositories,
            ManifestNode::Component { .. } => &mut result.components,
            ManifestNode::Object { .. } => &mut result.objects,
            ManifestNode::Interface { .. } => &mut result.interfaces,
            ManifestNode::Contract { .. } => &mut result.contracts,
            ManifestNode::Requirement { .. } => &mut result.requirements,
            ManifestNode::Test { .. } => &mut result.tests,
            ManifestNode::Review { .. } => &mut result.reviews,
        };
        *count = count.saturating_add(1);
    }
    Ok(result)
}

fn validate_counts(counts: ModelCounts, limits: EcosystemLimits) -> Result<(), ModelDecision> {
    let within_limits = counts.repositories <= limits.repositories
        && counts.components <= limits.components
        && counts.objects <= limits.objects
        && counts.interfaces <= limits.interfaces
        && counts.contracts <= limits.contracts
        && counts.requirements <= limits.requirements
        && counts.tests <= limits.tests
        && counts.reviews <= limits.reviews
        && counts.gaps <= limits.gaps
        && counts.edges <= limits.edges;
    if !within_limits {
        return Err(resource(
            "model.counts",
            "retained population exceeds an export ceiling",
        ));
    }
    Ok(())
}
