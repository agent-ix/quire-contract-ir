//! Strict reader and exact lowering model for QSpec I04 `CheckedPackage` V1.
//!
//! This module consumes the public schema frozen by quire-specification PR 66
//! (`cd4c0a0aff56ad97d1cf5379847c903dc2728dd5`).  It deliberately has no
//! dependency on the native parser, CST, or QSL's implementation-private
//! package types.

use super::common::{
    canonical_value, decode_closed, digest_bytes, digest_json, exceeds, is_digest, is_nonempty,
    validate_locked_artifact, validate_source_map_entries, validate_term, ArtifactDigests, Stop,
    TermGrammar, ValidationFailure, NODE_DOMAIN,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

/// The sole I04 transport version admitted by this reader.
pub const CHECKED_PACKAGE_V1: &str = "quire.checked-package/v1";
const IDENTITY_PREIMAGE_V1: &str = "quire.checked-package-id/v1";
const GRAPH_V1: &str = "quire.checked-semantic-graph/v1";
const PACKAGE_DOMAIN: &str = "quire.package.semantic/v1";

/// Caller-supplied, exact ceilings for a checked-package read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedPackageReadLimits {
    /// Maximum canonical wire bytes.
    pub bytes: u64,
    /// Maximum JSON nesting depth.
    pub depth: u64,
    /// Maximum semantic graph nodes.
    pub nodes: u64,
    /// Maximum graph dependency edges.
    pub edges: u64,
    /// Maximum semantic occurrences and source-map entries combined.
    pub occurrences: u64,
    /// Maximum diagnostic entries.
    pub diagnostics: u64,
    /// Maximum semantic-term validation visits.
    pub work: u64,
}

impl CheckedPackageReadLimits {
    /// A finite default appropriate for one local request.
    pub const fn bounded() -> Self {
        Self {
            bytes: 1 << 20,
            depth: 128,
            nodes: 10_000,
            edges: 100_000,
            occurrences: 100_000,
            diagnostics: 10_000,
            work: 1_000_000,
        }
    }
}

/// A named resource meter in the closed I04 incomplete outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedPackageLimit {
    /// Wire byte budget.
    Bytes,
    /// JSON nesting budget.
    Depth,
    /// Graph node budget.
    Nodes,
    /// Graph edge budget.
    Edges,
    /// Occurrence and source-map budget.
    Occurrences,
    /// Diagnostic budget.
    Diagnostics,
    /// Validation-work budget.
    Work,
}

/// Stable machine code for a refused I04 read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedPackageRefusalCode {
    /// An unrecognized contract version was supplied.
    UnknownContractVersion,
    /// The byte stream cannot be interpreted as the selected wire.
    MalformedWire,
    /// A decoded object repeated a member name.
    DuplicateMember,
    /// A decoded object carried a member outside the closed schema.
    UnknownMember,
    /// The parsed value was not RFC-8785-style canonical JSON bytes.
    NoncanonicalWire,
    /// A locked source, definition, or model did not match supplied bytes.
    StaleDependency,
    /// A digest appeared under an incompatible identity domain.
    DigestDomainMismatch,
    /// A required capability was absent or non-available.
    UnknownRequiredCapability,
    /// Graph identity, reference, form, or source correspondence was invalid.
    InvalidSemanticGraph,
    /// The source occurrence map was invalid or incomplete.
    InvalidSourceMap,
    /// The selected graph contained a tag unknown to V1.
    UnsupportedNodeTag,
}

/// A typed refusal with a stable code and structural path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackageRefusal {
    /// The condition callers can distinguish without parsing prose.
    pub code: CheckedPackageRefusalCode,
    /// The closed-schema path at which admission failed.
    pub path: Box<str>,
}

/// A typed non-conclusive outcome caused by the first exhausted limit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackageIncomplete {
    /// The first measured resource that could not be charged.
    pub limit_kind: CheckedPackageLimit,
    /// Caller-selected ceiling.
    pub limit: u64,
    /// Counter value at the failed charge.
    pub consumed: u64,
}

/// The closed result of reading untrusted I04 bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageReadResult {
    /// A complete checked package is safe for lowering.
    Admitted(Box<CheckedPackage>),
    /// The input is invalid for this version.
    Refused(CheckedPackageRefusal),
    /// The input may be valid but exceeded a caller limit.
    Incomplete(CheckedPackageIncomplete),
}

/// Exact source material used to prove a lock entry is not stale.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CheckedArtifactLocator {
    /// Artifact identity authority.
    pub authority: Box<str>,
    /// Artifact identity.
    pub identity: Box<str>,
    /// Revision namespace.
    pub revision_namespace: Box<str>,
    /// Revision value.
    pub revision_value: Box<str>,
    /// Identity domain of the bytes.
    pub domain: Box<str>,
}

/// Material supplied by the caller to validate package lock entries.
#[derive(Clone, Debug, Default)]
pub struct CheckedPackageReadContext {
    artifacts: BTreeMap<CheckedArtifactLocator, Vec<u8>>,
}

impl CheckedPackageReadContext {
    /// Starts an empty context. Every lock entry must be supplied for admission.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds immutable bytes under their complete artifact locator.
    pub fn insert(&mut self, locator: CheckedArtifactLocator, bytes: Vec<u8>) {
        self.artifacts.insert(locator, bytes);
    }
}

impl ArtifactDigests for CheckedPackageReadContext {
    fn artifact_digest(&self, locator: &CheckedArtifactLocator) -> Option<Cow<'_, str>> {
        self.artifacts
            .get(locator)
            .map(|bytes| Cow::Owned(digest_bytes(bytes)))
    }
}

/// A semantic digest with an explicit domain and algorithm.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSemanticId {
    /// Identity domain.
    pub domain: Box<str>,
    /// Digest algorithm.
    pub algorithm: Box<str>,
    /// Lowercase SHA-256 digest.
    pub digest: Box<str>,
}

/// A checked semantic graph node identity.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedNodeId {
    /// Must be `quire.checked-semantic-node/v1`.
    pub domain: Box<str>,
    /// Lowercase SHA-256 digest.
    pub digest: Box<str>,
}

/// A source occurrence role.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedOccurrenceRole {
    /// Declaration occurrence.
    Declaration,
    /// Type occurrence.
    Type,
    /// Expression occurrence.
    Expression,
    /// Anchor occurrence.
    Anchor,
    /// Claim occurrence.
    Claim,
    /// Generated occurrence.
    Generated,
}

/// One semantic occurrence named by a graph node.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOccurrence {
    /// Semantic role.
    pub role: CheckedOccurrenceRole,
    /// Stable ordinal for repeated roles.
    pub ordinal: u64,
}

/// One raw source, definition, or model identity from the package lock.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedArtifactRef {
    /// Artifact authority.
    pub authority: Box<str>,
    /// Artifact identity.
    pub identity: Box<str>,
    /// Artifact revision.
    pub revision: CheckedRevision,
    /// Typed byte-digest domain.
    pub digest_domain: Box<str>,
    /// Lowercase SHA-256 digest.
    pub digest: Box<str>,
    /// Model export name, present only for compiled-model references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export: Option<Box<str>>,
}

/// A revision in a stable namespace.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedRevision {
    /// Revision namespace.
    pub namespace: Box<str>,
    /// Revision value.
    pub value: Box<str>,
}

/// A selected definition-role identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSelection {
    /// The closed selection role.
    pub role: Box<str>,
    /// Selected definition artifact.
    pub definition: CheckedArtifactRef,
}

/// The exact immutable package lock.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageLock {
    /// Raw source documents.
    pub sources: Vec<CheckedArtifactRef>,
    /// Selected language edition.
    pub edition: CheckedSelection,
    /// Selected profiles.
    pub profile_selections: Vec<CheckedSelection>,
    /// Selected definitions.
    pub definition_selections: Vec<CheckedArtifactRef>,
    /// Selected compiled models.
    pub model_selections: Vec<CheckedArtifactRef>,
    /// Features required by the checked package.
    pub required_features: Vec<Box<str>>,
    /// Resolved dependency selections.
    pub dependency_selections: Vec<CheckedSelection>,
}

/// Identity projection of a graph node, excluding source occurrences.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedNodeProjection {
    /// Stable node identity.
    pub node_id: CheckedNodeId,
    /// Graph schema version.
    pub schema_version: Box<str>,
    /// Closed node family tag.
    pub node_tag: Box<str>,
    /// Closed form within its node family.
    pub semantic_form: Box<str>,
    /// Checked semantic type identity.
    pub semantic_type: CheckedNodeId,
    /// Identity-based dependencies.
    pub dependencies: Vec<CheckedNodeId>,
    /// Explicit recursion group when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursion_group: Option<Box<str>>,
    /// Typed semantic term encoded in the public contract.
    pub body: Value,
}

/// The non-circular package identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageIdentityPreimage {
    /// Must be `quire.checked-package-id/v1`.
    pub version: Box<str>,
    /// Selected edition.
    pub edition: CheckedSelection,
    /// Selected profiles.
    pub profile_selections: Vec<CheckedSelection>,
    /// Selected definitions.
    pub definition_selections: Vec<CheckedArtifactRef>,
    /// Selected models.
    pub model_selections: Vec<CheckedArtifactRef>,
    /// Required features.
    pub required_features: Vec<Box<str>>,
    /// Dependency selections.
    pub dependency_selections: Vec<CheckedSelection>,
    /// Ordered graph identity projection.
    pub identity_projection: Vec<CheckedNodeProjection>,
}

/// A checked semantic graph node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSemanticNode {
    /// Stable checked node identity.
    pub node_id: CheckedNodeId,
    /// Must be `quire.checked-semantic-graph/v1`.
    pub schema_version: Box<str>,
    /// Closed semantic family.
    pub node_tag: Box<str>,
    /// Closed semantic form for the family.
    pub semantic_form: Box<str>,
    /// Checked semantic type identity.
    pub semantic_type: CheckedNodeId,
    /// Identity-based dependencies.
    pub dependencies: Vec<CheckedNodeId>,
    /// Exact source occurrence keys.
    pub occurrences: Vec<CheckedOccurrence>,
    /// Explicit recursion group when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursion_group: Option<Box<str>>,
    /// Typed public semantic term.
    pub body: Value,
}

/// A checked graph and its selected version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSemanticGraph {
    /// Graph contract version.
    pub graph_version: Box<str>,
    /// Complete closed graph.
    pub nodes: Vec<CheckedSemanticNode>,
}

/// A source interval under an exact raw-source identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSourceRegion {
    /// Raw source document identity.
    pub source: CheckedArtifactRef,
    /// Inclusive byte start.
    pub start: u64,
    /// Exclusive byte end.
    pub end: u64,
}

/// Exact source correspondence for one node occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSourceMapEntry {
    /// Graph node identity.
    pub node_id: CheckedNodeId,
    /// Occurrence role.
    pub role: CheckedOccurrenceRole,
    /// Occurrence ordinal.
    pub ordinal: u64,
    /// One or more half-open source regions.
    pub regions: Vec<CheckedSourceRegion>,
}

/// Reported availability of one required capability.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedCapability {
    /// Capability identifier.
    pub feature: Box<str>,
    /// Producer/model disposition.
    pub disposition: Box<str>,
}

/// Typed package diagnostics, retained without message-derived classification.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDiagnostics {
    /// Diagnostic catalog definition.
    pub catalog: CheckedArtifactRef,
    /// Structured diagnostic records.
    pub entries: Vec<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedPackageWire {
    contract_version: Box<str>,
    identity_preimage: CheckedPackageIdentityPreimage,
    package_id: CheckedSemanticId,
    lock: CheckedPackageLock,
    semantic_graph: CheckedSemanticGraph,
    source_map: Vec<CheckedSourceMapEntry>,
    capability_report: Vec<CheckedCapability>,
    diagnostics: CheckedDiagnostics,
}

/// Immutable, admitted public checked-package data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackage {
    wire: CheckedPackageWire,
}

impl CheckedPackage {
    /// Reads one canonical I04 value without exposing a partial package.
    pub fn read(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        context: &CheckedPackageReadContext,
    ) -> CheckedPackageReadResult {
        match canonical_value(bytes, limits)
            .and_then(|value| Self::admit_value(value, limits, context))
        {
            Ok(package) => CheckedPackageReadResult::Admitted(Box::new(package)),
            Err(stop) => stop.into(),
        }
    }

    /// Decodes and validates an already canonical V1 value against digest
    /// evidence. Shared by [`CheckedPackage::read`] and the version dispatcher.
    pub(super) fn admit_value(
        value: Value,
        limits: CheckedPackageReadLimits,
        context: &dyn ArtifactDigests,
    ) -> Result<Self, Stop> {
        let wire = decode_closed::<CheckedPackageWire>(value)?;
        let package = Self { wire };
        package.validate(limits, context)?;
        Ok(package)
    }

    /// The exact admitted lock, for migration within this module tree.
    pub(super) fn lock(&self) -> &CheckedPackageLock {
        &self.wire.lock
    }

    /// Returns the versioned semantic package identity.
    pub fn package_id(&self) -> &CheckedSemanticId {
        &self.wire.package_id
    }

    /// Returns the complete closed checked graph.
    pub fn graph(&self) -> &CheckedSemanticGraph {
        &self.wire.semantic_graph
    }

    /// Lowers independently requested nodes, preserving every sibling outcome.
    pub fn lower(&self, requested: &[CheckedNodeId], limit: u64) -> CompleteLoweringResult {
        let nodes = self
            .wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        let mut work = 0_u64;
        let mut records = Vec::with_capacity(requested.len());
        for request in requested {
            work = work.saturating_add(1);
            if work > limit {
                records.push(CompleteLoweringRecord::Failed {
                    node_id: request.clone(),
                    limit,
                    consumed: work,
                });
                continue;
            }
            let Some(node) = nodes.get(request) else {
                records.push(CompleteLoweringRecord::InvalidInput {
                    node_id: request.clone(),
                });
                continue;
            };
            records.push(CompleteLoweringRecord::Lowered {
                node: CompleteContractNode::from_checked(node, &self.wire.source_map),
            });
        }
        CompleteLoweringResult {
            package_id: self.wire.package_id.clone(),
            records,
        }
    }

    fn validate(
        &self,
        limits: CheckedPackageReadLimits,
        context: &dyn ArtifactDigests,
    ) -> Result<(), ValidationFailure> {
        if self.wire.contract_version.as_ref() != CHECKED_PACKAGE_V1 {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::UnknownContractVersion,
                "contract_version",
            ));
        }
        if self.wire.package_id.domain.as_ref() != PACKAGE_DOMAIN {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::DigestDomainMismatch,
                "package_id.domain",
            ));
        }
        if self.wire.package_id.algorithm.as_ref() != "sha256"
            || !is_digest(&self.wire.package_id.digest)
        {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::MalformedWire,
                "package_id",
            ));
        }
        if self.wire.identity_preimage.version.as_ref() != IDENTITY_PREIMAGE_V1 {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::MalformedWire,
                "identity_preimage.version",
            ));
        }
        let preimage = serde_json::to_value(&self.wire.identity_preimage).map_err(|_| {
            ValidationFailure::Refused(
                CheckedPackageRefusalCode::MalformedWire,
                "identity_preimage",
            )
        })?;
        let computed_digest = digest_json(&preimage).map_err(|_| {
            ValidationFailure::Refused(
                CheckedPackageRefusalCode::MalformedWire,
                "identity_preimage",
            )
        })?;
        if computed_digest != self.wire.package_id.digest.as_ref() {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::StaleDependency,
                "package_id.digest",
            ));
        }
        self.validate_lock(context)?;
        self.validate_graph(limits)?;
        self.validate_source_map(limits, context)?;
        self.validate_capabilities()?;
        let diagnostics = u64::try_from(self.wire.diagnostics.entries.len()).unwrap_or(u64::MAX);
        if exceeds(diagnostics, limits.diagnostics) {
            return Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Diagnostics,
                limits.diagnostics,
                diagnostics,
            ));
        }
        Ok(())
    }

    fn validate_lock(&self, context: &dyn ArtifactDigests) -> Result<(), ValidationFailure> {
        let lock = &self.wire.lock;
        if lock.sources.is_empty() || !same_non_graph_lock(&self.wire.identity_preimage, lock) {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::StaleDependency,
                "lock",
            ));
        }
        for source in &lock.sources {
            validate_locked_artifact(source, "quire.source.bytes/v1", context, "lock.sources")?;
        }
        validate_locked_artifact(
            &lock.edition.definition,
            "quire.definition.bytes/v1",
            context,
            "lock.edition",
        )?;
        for selection in lock
            .profile_selections
            .iter()
            .chain(lock.dependency_selections.iter())
        {
            validate_locked_artifact(
                &selection.definition,
                "quire.definition.bytes/v1",
                context,
                "lock.selection",
            )?;
        }
        for definition in &lock.definition_selections {
            validate_locked_artifact(
                definition,
                "quire.definition.bytes/v1",
                context,
                "lock.definition_selections",
            )?;
        }
        for model in &lock.model_selections {
            validate_locked_artifact(
                model,
                "quire.compiled-model.bytes/v1",
                context,
                "lock.model_selections",
            )?;
            if model.export.as_deref().is_none_or(str::is_empty) {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::MalformedWire,
                    "lock.model_selections.export",
                ));
            }
        }
        validate_locked_artifact(
            &self.wire.diagnostics.catalog,
            "quire.definition.bytes/v1",
            context,
            "diagnostics.catalog",
        )
    }

    fn validate_graph(&self, limits: CheckedPackageReadLimits) -> Result<(), ValidationFailure> {
        let graph = &self.wire.semantic_graph;
        if graph.graph_version.as_ref() != GRAPH_V1 || graph.nodes.is_empty() {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph",
            ));
        }
        let node_count = u64::try_from(graph.nodes.len()).unwrap_or(u64::MAX);
        if exceeds(node_count, limits.nodes) {
            return Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Nodes,
                limits.nodes,
                node_count,
            ));
        }
        let mut ids = BTreeSet::new();
        let mut work = 0_u64;
        let mut edges = 0_u64;
        for node in &graph.nodes {
            if node.node_id.domain.as_ref() != NODE_DOMAIN || !is_digest(&node.node_id.digest) {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::DigestDomainMismatch,
                    "semantic_graph.nodes.node_id",
                ));
            }
            if !ids.insert(node.node_id.clone()) {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    "semantic_graph.nodes.node_id",
                ));
            }
            if node.schema_version.as_ref() != GRAPH_V1 {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    "semantic_graph.nodes.schema_version",
                ));
            }
            validate_form(&node.node_tag, &node.semantic_form)?;
            if node.semantic_type.domain.as_ref() != NODE_DOMAIN
                || !is_digest(&node.semantic_type.digest)
            {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::DigestDomainMismatch,
                    "semantic_graph.nodes.semantic_type",
                ));
            }
            edges =
                edges.saturating_add(u64::try_from(node.dependencies.len()).unwrap_or(u64::MAX));
            if exceeds(edges, limits.edges) {
                return Err(ValidationFailure::Incomplete(
                    CheckedPackageLimit::Edges,
                    limits.edges,
                    edges,
                ));
            }
            work = work.saturating_add(validate_term(&node.body, TermGrammar::V1, &mut |_| ())?);
            if exceeds(work, limits.work) {
                return Err(ValidationFailure::Incomplete(
                    CheckedPackageLimit::Work,
                    limits.work,
                    work,
                ));
            }
            if node.occurrences.is_empty() {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    "semantic_graph.nodes.occurrences",
                ));
            }
        }
        for node in &graph.nodes {
            if !ids.contains(&node.semantic_type)
                || node
                    .dependencies
                    .iter()
                    .any(|dependency| !ids.contains(dependency))
            {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    "semantic_graph.nodes.dependencies",
                ));
            }
        }
        let projection = graph.nodes.iter().map(node_projection).collect::<Vec<_>>();
        if projection != self.wire.identity_preimage.identity_projection {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::StaleDependency,
                "identity_preimage.identity_projection",
            ));
        }
        Ok(())
    }

    fn validate_source_map(
        &self,
        limits: CheckedPackageReadLimits,
        context: &dyn ArtifactDigests,
    ) -> Result<(), ValidationFailure> {
        validate_source_map_entries(
            self.wire
                .semantic_graph
                .nodes
                .iter()
                .map(|node| (&node.node_id, node.occurrences.as_slice())),
            &self.wire.source_map,
            &self.wire.lock.sources,
            limits,
            context,
        )
    }

    fn validate_capabilities(&self) -> Result<(), ValidationFailure> {
        let mut reported = BTreeMap::new();
        for capability in &self.wire.capability_report {
            if !is_nonempty(&capability.feature)
                || !reported
                    .insert(capability.feature.as_ref(), capability.disposition.as_ref())
                    .is_none()
            {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::UnknownRequiredCapability,
                    "capability_report",
                ));
            }
        }
        for feature in &self.wire.lock.required_features {
            if reported.get(feature.as_ref()) != Some(&"available") {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::UnknownRequiredCapability,
                    "capability_report",
                ));
            }
        }
        Ok(())
    }
}

/// A core Contract IR node obtained by exact lowering of an admitted I04 node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteContractNode {
    /// Stable semantic identity.
    pub node_id: CheckedNodeId,
    /// Closed source semantic family.
    pub node_tag: Box<str>,
    /// Exact source semantic form.
    pub semantic_form: Box<str>,
    /// Semantic type identity.
    pub semantic_type: CheckedNodeId,
    /// Stable dependency identities.
    pub dependencies: Vec<CheckedNodeId>,
    /// Typed public semantic term.
    pub body: Value,
    /// Exact source correspondence.
    pub source_map: Vec<CheckedSourceMapEntry>,
}

impl CompleteContractNode {
    fn from_checked(node: &CheckedSemanticNode, source_map: &[CheckedSourceMapEntry]) -> Self {
        Self {
            node_id: node.node_id.clone(),
            node_tag: node.node_tag.clone(),
            semantic_form: node.semantic_form.clone(),
            semantic_type: node.semantic_type.clone(),
            dependencies: node.dependencies.clone(),
            body: node.body.clone(),
            source_map: source_map
                .iter()
                .filter(|entry| entry.node_id == node.node_id)
                .cloned()
                .collect(),
        }
    }
}

/// One terminal per-request complete-V1 lowering outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompleteLoweringRecord {
    /// Exact semantic node was lowered.
    Lowered { node: CompleteContractNode },
    /// Requested node identity was absent from the admitted graph.
    InvalidInput { node_id: CheckedNodeId },
    /// This individual request exceeded lowering work.
    Failed {
        node_id: CheckedNodeId,
        limit: u64,
        consumed: u64,
    },
}

/// Independent per-item outcomes for one package identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteLoweringResult {
    /// Identity of the admitted source package.
    pub package_id: CheckedSemanticId,
    /// One record per requested node, in request order.
    pub records: Vec<CompleteLoweringRecord>,
}

fn same_non_graph_lock(
    preimage: &CheckedPackageIdentityPreimage,
    lock: &CheckedPackageLock,
) -> bool {
    preimage.edition == lock.edition
        && preimage.profile_selections == lock.profile_selections
        && preimage.definition_selections == lock.definition_selections
        && preimage.model_selections == lock.model_selections
        && preimage.required_features == lock.required_features
        && preimage.dependency_selections == lock.dependency_selections
}

fn node_projection(node: &CheckedSemanticNode) -> CheckedNodeProjection {
    CheckedNodeProjection {
        node_id: node.node_id.clone(),
        schema_version: node.schema_version.clone(),
        node_tag: node.node_tag.clone(),
        semantic_form: node.semantic_form.clone(),
        semantic_type: node.semantic_type.clone(),
        dependencies: node.dependencies.clone(),
        recursion_group: node.recursion_group.clone(),
        body: node.body.clone(),
    }
}

fn validate_form(tag: &str, form: &str) -> Result<(), ValidationFailure> {
    let forms = match tag {
        "scalar_type" => &[
            "boolean",
            "integer",
            "rational",
            "decimal",
            "float32",
            "float64",
            "text",
            "dimension",
            "unit",
            "enum",
        ][..],
        "composite_type" => &[
            "option",
            "sequence",
            "set",
            "bag",
            "ordered_set",
            "record",
            "tuple",
            "alias",
            "reference",
        ][..],
        "bounded_domain" => &[
            "integer_range",
            "rational_range",
            "decimal_range",
            "float_rounding",
            "text_bounds",
            "collection_bounds",
            "model_population",
        ][..],
        "value" => &[
            "literal",
            "enum_value",
            "collection_value",
            "record_value",
            "tuple_value",
            "option_value",
        ][..],
        "expression" => &[
            "reference",
            "call",
            "unary",
            "binary",
            "conditional",
            "let",
            "quantify",
            "conversion",
            "query",
            "pre_read",
            "presence_read",
            "deref",
            "reachability",
        ][..],
        "function" => &["pure_function", "predicate", "recursive_function"][..],
        "model" => &[
            "model_import",
            "model_export",
            "model_type",
            "model_declaration",
        ][..],
        "relation" => &[
            "relationship",
            "population",
            "membership",
            "causal_relation",
        ][..],
        "state" => &[
            "state_clause",
            "frame",
            "transition",
            "operation_anchor",
            "snapshot",
        ][..],
        "temporal" => &[
            "temporal_clause",
            "formula",
            "clock",
            "window",
            "activation",
            "deadline",
        ][..],
        "protocol" => &[
            "protocol_clause",
            "role",
            "channel",
            "queue",
            "control",
            "obligation",
            "compensation",
        ][..],
        "claim" => &[
            "verification_claim",
            "analysis_claim",
            "hyperproperty",
            "synthesis_request",
        ][..],
        "correspondence" => &[
            "source_locus",
            "model_correspondence",
            "binding_role",
            "profile_correspondence",
        ][..],
        _ => {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::UnsupportedNodeTag,
                "semantic_graph.nodes.node_tag",
            ))
        }
    };
    if forms.contains(&form) {
        Ok(())
    } else {
        Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.semantic_form",
        ))
    }
}
