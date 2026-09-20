//! Version-neutral I04 `CheckedPackage` vocabulary.
//!
//! These types carry no contract-version-specific shape of their own: they
//! are the caller-facing limits, refusal/incomplete outcomes, and locked
//! source/occurrence records that the current reader and its lowering
//! pipeline both use unchanged.

use serde::{Deserialize, Serialize};

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
    /// The selected graph contained a tag unknown to the current contract.
    UnsupportedNodeTag,
    /// A frame entry named no declared dependency of its frame node.
    MissingDeclaration,
    /// A frame entry named a declared dependency of a meaning its member
    /// does not admit.
    InvalidModelBinding,
    /// An application node's `operation` failed catalog validation, or its
    /// retained key is not its own preimage digest.
    InvalidPackage,
    /// An application node's argument, member or leaf is ill-typed against
    /// its catalogued operation.
    IllTyped,
}

/// Stable machine cause paired with a [`CheckedPackageRefusalCode`] under
/// FR-322's closed `DiagnosticCausePairing` (`schema.json`). Only the causes
/// this reader currently produces; FR-322's remaining cause tags — ambiguous
/// declarations and `declaration-nominal-mismatch` (the README names it a
/// reader-stage cause; this reader does not produce it) — belong to stages
/// this reader does not yet implement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedPackageRefusalCause {
    /// `missing-name`: an entry naming no node of the graph, or a node that
    /// is not a declared dependency of the frame node that names it.
    MissingName,
    /// `malformed-declaration`: an entry naming a declared dependency whose
    /// tag and semantic form its member does not admit.
    MalformedDeclaration,
    /// `stale-node-key`: an application node whose retained key is not the
    /// JCS SHA-256 of its own preimage.
    StaleNodeKey,
    /// `unknown-operation`: an `operation.identity` absent from the catalog.
    UnknownOperation,
    /// `operation-class-mismatch`: the application's `operator` does not
    /// equal the catalogued entry's operator class.
    OperationClassMismatch,
    /// `operation-law-missing`: `operation.laws` (or a required `leaves`
    /// entry) is shorter than the catalogued entry requires.
    OperationLawMissing,
    /// `operation-law-mismatch`: a declared law names a role or definition
    /// the catalogued entry (or its law-role table) does not admit.
    OperationLawMismatch,
    /// `operation-law-unselected`: a declared law's definition is a
    /// catalogued member of its role but is absent from the package lock's
    /// own selections.
    OperationLawUnselected,
    /// `operation-mode-mismatch`: `operation.mode`'s presence or kind
    /// disagrees with the catalogued entry's mode kind.
    OperationModeMismatch,
    /// `operation-mode-type-mismatch`: `operation.mode`'s value disagrees
    /// with the value an operand or leaf's own type pins.
    OperationModeTypeMismatch,
    /// `operation-member-mismatch`: `operation.member`'s presence or kind
    /// disagrees with the catalogued entry's member kind.
    OperationMemberMismatch,
    /// `operator-ineligible`: an argument's arity, type or named member does
    /// not fit the catalogued entry's operands, constraints or member.
    OperatorIneligible,
}

/// A typed refusal with a stable code and structural path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackageRefusal {
    /// The condition callers can distinguish without parsing prose.
    pub code: CheckedPackageRefusalCode,
    /// The closed-schema path at which admission failed.
    pub path: Box<str>,
    /// The cause tag paired with `code` under FR-322's `DiagnosticCausePairing`,
    /// present exactly when this reader determined one.
    pub cause: Option<CheckedPackageRefusalCause>,
    /// The node key of the offending entry or node, present whenever this
    /// reader located the refusal at a specific graph node.
    pub locus: Option<CheckedNodeId>,
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

/// A revision in a stable namespace.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedRevision {
    /// Revision namespace.
    pub namespace: Box<str>,
    /// Revision value.
    pub value: Box<str>,
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

/// A selected definition-role identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSelection {
    /// The closed selection role.
    pub role: Box<str>,
    /// Selected definition artifact.
    pub definition: CheckedArtifactRef,
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
