//! Strict manifest contract and exact campaign admission.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::bridge::{
    canonical, BridgeDigest, BridgeError, BridgeErrorCode, BridgeLimits, ContractSelection,
};

use super::decision::{ModelCauseCode as Code, ModelDecision};
use super::graph::{self, ValidatedGraph};
use super::{MANIFEST_ID_PROFILE, MANIFEST_PROFILE, MANIFEST_SCHEMA_SHA256};

/// The exact repository identities in the temporal ecosystem.
pub const REPOSITORY_IDENTITIES: [&str; 9] = [
    "agent-ix/quire-contract-ir",
    "agent-ix/quire-observation",
    "agent-ix/quire-protocol",
    "agent-ix/quire-spec-language",
    "agent-ix/quire-specification",
    "agent-ix/tl-mltl",
    "agent-ix/tl-parse",
    "agent-ix/tl-rewrite",
    "agent-ix/tl-syntax",
];

/// Owner maxima for the descriptive ecosystem model.
pub const OWNER_MAX: EcosystemLimits = EcosystemLimits {
    manifest_bytes: 64 * 1024 * 1024,
    model_bytes: 64 * 1024 * 1024,
    json_depth: 256,
    string_bytes: 1024 * 1024,
    repositories: 9,
    components: 256,
    objects: 10_000,
    interfaces: 10_000,
    contracts: 10_000,
    requirements: 10_000,
    tests: 10_000,
    reviews: 10_000,
    gaps: 10_000,
    edges: 100_000,
    visited_work: 128 * 1024 * 1024,
    allocation_bytes: 128 * 1024 * 1024,
};

/// Caller-lowerable, platform-independent model ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EcosystemLimits {
    /// Maximum canonical input-manifest bytes.
    pub manifest_bytes: u64,
    /// Maximum canonical exported-model bytes.
    pub model_bytes: u64,
    /// Maximum JSON nesting depth accepted or emitted.
    pub json_depth: u64,
    /// Maximum UTF-8 bytes in any JSON string or object key.
    pub string_bytes: u64,
    /// Maximum repository-node population.
    pub repositories: u64,
    /// Maximum component-node population.
    pub components: u64,
    /// Maximum semantic-object-node population.
    pub objects: u64,
    /// Maximum executable-interface-node population.
    pub interfaces: u64,
    /// Maximum exact-contract-selection-node population.
    pub contracts: u64,
    /// Maximum requirement-node population.
    pub requirements: u64,
    /// Maximum test-node population.
    pub tests: u64,
    /// Maximum review-node population.
    pub reviews: u64,
    /// Maximum unresolved-gap population.
    pub gaps: u64,
    /// Maximum typed-edge population.
    pub edges: u64,
    /// Maximum deterministic decoding and graph-traversal work.
    pub visited_work: u64,
    /// Deterministic reservation failpoint used before retained allocation.
    pub allocation_bytes: u64,
}

impl EcosystemLimits {
    /// Intersects each semantic ceiling with the immutable owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            manifest_bytes: min(self.manifest_bytes, OWNER_MAX.manifest_bytes),
            model_bytes: min(self.model_bytes, OWNER_MAX.model_bytes),
            json_depth: min(self.json_depth, OWNER_MAX.json_depth),
            string_bytes: min(self.string_bytes, OWNER_MAX.string_bytes),
            repositories: min(self.repositories, OWNER_MAX.repositories),
            components: min(self.components, OWNER_MAX.components),
            objects: min(self.objects, OWNER_MAX.objects),
            interfaces: min(self.interfaces, OWNER_MAX.interfaces),
            contracts: min(self.contracts, OWNER_MAX.contracts),
            requirements: min(self.requirements, OWNER_MAX.requirements),
            tests: min(self.tests, OWNER_MAX.tests),
            reviews: min(self.reviews, OWNER_MAX.reviews),
            gaps: min(self.gaps, OWNER_MAX.gaps),
            edges: min(self.edges, OWNER_MAX.edges),
            visited_work: min(self.visited_work, OWNER_MAX.visited_work),
            allocation_bytes: min(self.allocation_bytes, OWNER_MAX.allocation_bytes),
        }
    }
}

impl Default for EcosystemLimits {
    fn default() -> Self {
        OWNER_MAX
    }
}

/// One exact repository expected by the campaign reader.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExpectedRepository {
    identity: Box<str>,
    revision: Box<str>,
}

impl ExpectedRepository {
    /// Selects one immutable merged repository revision.
    #[must_use]
    pub fn new(identity: impl Into<Box<str>>, revision: impl Into<Box<str>>) -> Self {
        Self {
            identity: identity.into(),
            revision: revision.into(),
        }
    }
}

/// Exact external campaign authority used to admit one manifest.
#[derive(Clone, Debug)]
pub struct ExpectedCampaign {
    campaign: Box<str>,
    repositories: Vec<ExpectedRepository>,
    manifest_digest: BridgeDigest,
}

impl ExpectedCampaign {
    /// Constructs an expectation for exactly nine immutable repositories.
    #[must_use]
    pub fn new(
        campaign: impl Into<Box<str>>,
        repositories: [ExpectedRepository; 9],
        manifest_digest: BridgeDigest,
    ) -> Self {
        let mut repositories = Vec::from(repositories);
        repositories.sort();
        Self {
            campaign: campaign.into(),
            repositories,
            manifest_digest,
        }
    }

    /// Returns the selected campaign identity.
    #[must_use]
    pub fn campaign(&self) -> &str {
        &self.campaign
    }

    /// Returns SHA-256 of the exact selected canonical manifest bytes.
    #[must_use]
    pub const fn manifest_digest(&self) -> BridgeDigest {
        self.manifest_digest
    }
}

/// One closed node in an authored ecosystem manifest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ManifestNode {
    Repository {
        identity: String,
        revision: String,
    },
    Component {
        identity: String,
    },
    Object {
        identity: String,
        revision: String,
    },
    Interface {
        identity: String,
        revision: String,
    },
    Contract {
        identity: String,
        selection: ContractSelection,
    },
    Requirement {
        identity: String,
        revision: String,
    },
    Test {
        identity: String,
        revision: String,
    },
    Review {
        identity: String,
        revision: String,
    },
}

impl ManifestNode {
    /// Returns the globally unique node identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        match self {
            Self::Repository { identity, .. }
            | Self::Component { identity }
            | Self::Object { identity, .. }
            | Self::Interface { identity, .. }
            | Self::Contract { identity, .. }
            | Self::Requirement { identity, .. }
            | Self::Test { identity, .. }
            | Self::Review { identity, .. } => identity,
        }
    }

    /// Returns the closed wire kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Repository { .. } => "repository",
            Self::Component { .. } => "component",
            Self::Object { .. } => "object",
            Self::Interface { .. } => "interface",
            Self::Contract { .. } => "contract",
            Self::Requirement { .. } => "requirement",
            Self::Test { .. } => "test",
            Self::Review { .. } => "review",
        }
    }

    pub(crate) fn sort_key(&self) -> (&'static str, &str) {
        (self.kind(), self.identity())
    }
}

/// Closed relation vocabulary for authored ecosystem edges.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManifestEdgeKind {
    Consumes,
    NormativeReference,
    Owns,
    RuntimeDependency,
    Verifies,
}

/// One typed directed manifest relation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestEdge {
    pub kind: ManifestEdgeKind,
    pub source: String,
    pub target: String,
}

/// One unresolved descriptive gap retained by the model.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestGap {
    pub identity: String,
    pub requirement: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManifestWire {
    pub profile: String,
    pub schema_digest: String,
    pub campaign: String,
    pub nodes: Vec<ManifestNode>,
    pub edges: Vec<ManifestEdge>,
    pub gaps: Vec<ManifestGap>,
}

/// Constructor-private manifest proven against exact campaign authority.
///
/// External callers cannot forge this proof with a struct literal:
///
/// ```compile_fail
/// use quire_contract_ir::ecosystem_model::CheckedManifestSet;
///
/// fn forge() -> CheckedManifestSet {
///     CheckedManifestSet {}
/// }
/// ```
#[derive(Clone, Debug)]
pub struct CheckedManifestSet {
    pub(crate) wire: ManifestWire,
    pub(crate) identity: BridgeDigest,
    pub(crate) bytes: Box<[u8]>,
    pub(crate) graph: ValidatedGraph,
    pub(crate) byte_digest: BridgeDigest,
}

impl CheckedManifestSet {
    /// Returns the exact canonical manifest bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the domain-separated manifest content identity.
    #[must_use]
    pub const fn identity(&self) -> BridgeDigest {
        self.identity
    }

    /// Returns SHA-256 of the complete selected manifest bytes.
    #[must_use]
    pub const fn byte_digest(&self) -> BridgeDigest {
        self.byte_digest
    }

    /// Returns the exact selected campaign.
    #[must_use]
    pub fn campaign(&self) -> &str {
        &self.wire.campaign
    }

    /// Returns the admitted sorted nodes.
    #[must_use]
    pub fn nodes(&self) -> &[ManifestNode] {
        &self.wire.nodes
    }

    /// Returns the admitted sorted typed edges.
    #[must_use]
    pub fn edges(&self) -> &[ManifestEdge] {
        &self.wire.edges
    }

    /// Returns unresolved descriptive gaps.
    #[must_use]
    pub fn gaps(&self) -> &[ManifestGap] {
        &self.wire.gaps
    }
}

/// Strict-reads and validates one exact campaign manifest.
pub fn read(
    bytes: &[u8],
    expected: &ExpectedCampaign,
    limits: EcosystemLimits,
) -> Result<CheckedManifestSet, ModelDecision> {
    let limits = limits.effective();
    if u64_len(bytes.len())? > limits.manifest_bytes
        || u64_len(bytes.len())? > limits.allocation_bytes
    {
        return Err(resource("manifest", "manifest byte ceiling exceeded"));
    }
    let wire: ManifestWire = canonical::decode(bytes, bridge_limits(limits, true))
        .map_err(|error| map_canonical(error, "manifest"))?;
    if wire.profile != MANIFEST_PROFILE || wire.schema_digest != MANIFEST_SCHEMA_SHA256 {
        return Err(ModelDecision::new(
            Code::ContractMismatch,
            "manifest.contract",
            "manifest profile or schema digest differs",
        ));
    }
    if wire.campaign != expected.campaign.as_ref() {
        return Err(ModelDecision::new(
            Code::CampaignMismatch,
            "manifest.campaign",
            "manifest campaign differs from the exact expectation",
        ));
    }
    validate_populations(&wire, expected, limits)?;
    let byte_digest = BridgeDigest::raw(bytes);
    if byte_digest != expected.manifest_digest {
        return Err(ModelDecision::new(
            Code::CampaignMismatch,
            "manifest.digest",
            "manifest bytes differ from the exact selected campaign",
        ));
    }
    let graph = graph::validate(&wire.nodes, &wire.edges, &wire.gaps, limits)?;
    let identity = BridgeDigest::domain(MANIFEST_ID_PROFILE, bytes);
    let mut retained = Vec::new();
    retained
        .try_reserve_exact(bytes.len())
        .map_err(|_| resource("manifest", "manifest byte reservation failed"))?;
    retained.extend_from_slice(bytes);
    Ok(CheckedManifestSet {
        wire,
        identity,
        bytes: retained.into_boxed_slice(),
        graph,
        byte_digest,
    })
}

fn validate_populations(
    wire: &ManifestWire,
    expected: &ExpectedCampaign,
    limits: EcosystemLimits,
) -> Result<(), ModelDecision> {
    let mut counts = [0_u64; 8];
    let mut seen = BTreeSet::new();
    let mut previous = None;
    let mut actual_repositories = Vec::new();
    for node in &wire.nodes {
        validate_text(node.identity(), limits, "manifest.nodes.identity")?;
        let key = node.sort_key();
        if previous.is_some_and(|prior| prior >= key) {
            let code = if seen.contains(node.identity()) {
                Code::DuplicateNode
            } else {
                Code::PopulationOutOfOrder
            };
            return Err(ModelDecision::new(
                code,
                node.identity(),
                "node population is not sorted and distinct",
            ));
        }
        if !seen.insert(node.identity()) {
            return Err(ModelDecision::new(
                Code::DuplicateNode,
                node.identity(),
                "duplicate node identity",
            ));
        }
        previous = Some(key);
        let index = match node {
            ManifestNode::Repository { identity, revision } => {
                validate_revision(revision, identity)?;
                actual_repositories.push(ExpectedRepository::new(
                    identity.as_str(),
                    revision.as_str(),
                ));
                0
            }
            ManifestNode::Component { .. } => 1,
            ManifestNode::Object { identity, revision } => {
                validate_revision(revision, identity)?;
                2
            }
            ManifestNode::Interface { identity, revision } => {
                validate_revision(revision, identity)?;
                3
            }
            ManifestNode::Contract {
                identity,
                selection,
            } => {
                if !selection.structurally_valid() {
                    return Err(ModelDecision::new(
                        Code::MovingRevision,
                        identity.as_str(),
                        "contract selection is not immutable",
                    ));
                }
                4
            }
            ManifestNode::Requirement { identity, revision } => {
                validate_revision(revision, identity)?;
                5
            }
            ManifestNode::Test { identity, revision } => {
                validate_revision(revision, identity)?;
                6
            }
            ManifestNode::Review { identity, revision } => {
                validate_revision(revision, identity)?;
                7
            }
        };
        counts[index] = counts[index].saturating_add(1);
    }
    let maxima = [
        limits.repositories,
        limits.components,
        limits.objects,
        limits.interfaces,
        limits.contracts,
        limits.requirements,
        limits.tests,
        limits.reviews,
    ];
    if counts
        .iter()
        .zip(maxima)
        .any(|(count, maximum)| *count > maximum)
    {
        return Err(resource(
            "manifest.nodes",
            "node-kind population ceiling exceeded",
        ));
    }
    actual_repositories.sort();
    let identities: Vec<_> = actual_repositories
        .iter()
        .map(|repository| repository.identity.as_ref())
        .collect();
    if identities != REPOSITORY_IDENTITIES {
        return Err(ModelDecision::new(
            Code::RepositorySetMismatch,
            "manifest.repositories",
            "repository set is not the closed nine-repository ecosystem",
        ));
    }
    if actual_repositories != expected.repositories {
        return Err(ModelDecision::new(
            Code::MovingRevision,
            "manifest.repositories",
            "repository revisions differ from the exact campaign expectation",
        ));
    }
    if u64_len(wire.edges.len())? > limits.edges || u64_len(wire.gaps.len())? > limits.gaps {
        return Err(resource(
            "manifest.graph",
            "edge or gap population ceiling exceeded",
        ));
    }
    if !wire.edges.windows(2).all(|rows| rows[0] < rows[1]) {
        let code = if wire.edges.windows(2).any(|rows| rows[0] == rows[1]) {
            Code::DuplicateEdge
        } else {
            Code::PopulationOutOfOrder
        };
        return Err(ModelDecision::new(
            code,
            "manifest.edges",
            "edge population is not sorted and distinct",
        ));
    }
    let mut gap_identities = BTreeSet::new();
    for gap in &wire.gaps {
        validate_text(&gap.identity, limits, "manifest.gaps.identity")?;
        validate_text(&gap.requirement, limits, "manifest.gaps.requirement")?;
        validate_text(&gap.description, limits, "manifest.gaps.description")?;
        if !gap_identities.insert(gap.identity.as_str()) {
            return Err(ModelDecision::new(
                Code::DuplicateGap,
                gap.identity.as_str(),
                "duplicate gap identity",
            ));
        }
    }
    if !wire.gaps.windows(2).all(|rows| rows[0] < rows[1]) {
        let code = if wire.gaps.windows(2).any(|rows| rows[0] == rows[1]) {
            Code::DuplicateGap
        } else {
            Code::PopulationOutOfOrder
        };
        return Err(ModelDecision::new(
            code,
            "manifest.gaps",
            "gap population is not sorted and distinct",
        ));
    }
    let work = u64_len(wire.nodes.len())?
        .saturating_add(u64_len(wire.edges.len())?)
        .saturating_add(u64_len(wire.gaps.len())?)
        .saturating_add(u64_len(wire.bytes_estimate())?);
    if work > limits.visited_work {
        return Err(resource("manifest", "visited-work ceiling exceeded"));
    }
    Ok(())
}

impl ManifestWire {
    fn bytes_estimate(&self) -> usize {
        self.campaign
            .len()
            .saturating_add(self.nodes.iter().map(|node| node.identity().len()).sum())
            .saturating_add(
                self.edges
                    .iter()
                    .map(|edge| edge.source.len().saturating_add(edge.target.len()))
                    .sum(),
            )
            .saturating_add(
                self.gaps
                    .iter()
                    .map(|gap| {
                        gap.identity
                            .len()
                            .saturating_add(gap.requirement.len())
                            .saturating_add(gap.description.len())
                    })
                    .sum(),
            )
    }
}

pub(crate) fn bridge_limits(limits: EcosystemLimits, manifest: bool) -> BridgeLimits {
    BridgeLimits {
        document_bytes: usize_limit(if manifest {
            limits.manifest_bytes
        } else {
            limits.model_bytes
        }),
        json_depth: usize_limit(limits.json_depth),
        string_bytes: usize_limit(limits.string_bytes),
        visited_work: usize_limit(limits.visited_work),
        allocation_bytes: usize_limit(limits.allocation_bytes),
        ..BridgeLimits::default()
    }
}

pub(crate) fn resource(path: impl Into<Box<str>>, detail: impl Into<Box<str>>) -> ModelDecision {
    ModelDecision::new(Code::ResourceExhausted, path, detail)
}

pub(crate) fn invalid(path: impl Into<Box<str>>, detail: impl Into<Box<str>>) -> ModelDecision {
    ModelDecision::new(Code::InvalidDocument, path, detail)
}

pub(crate) fn map_canonical(error: BridgeError, path: &'static str) -> ModelDecision {
    if error.code() == BridgeErrorCode::PredicateProjectionResourceExhausted {
        resource(path, error.message())
    } else {
        invalid(path, error.message())
    }
}

pub(crate) fn validate_text(
    value: &str,
    limits: EcosystemLimits,
    path: &'static str,
) -> Result<(), ModelDecision> {
    if value.is_empty()
        || u64_len(value.len())? > limits.string_bytes
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(invalid(
            path,
            "text is empty, contains controls, or exceeds the selected ceiling",
        ));
    }
    Ok(())
}

fn validate_revision(value: &str, path: &str) -> Result<(), ModelDecision> {
    if value.len() != 40
        || value
            .bytes()
            .any(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(ModelDecision::new(
            Code::MovingRevision,
            path,
            "revision is not an exact 40-character lowercase commit OID",
        ));
    }
    Ok(())
}

pub(crate) fn u64_len(value: usize) -> Result<u64, ModelDecision> {
    u64::try_from(value)
        .map_err(|_| resource("length", "length cannot be represented by the wire profile"))
}

fn usize_limit(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

const fn min(left: u64, right: u64) -> u64 {
    if left < right {
        left
    } else {
        right
    }
}
