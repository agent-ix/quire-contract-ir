//! Version-neutral I04 `CheckedPackage` vocabulary.
//!
//! These types carry no contract-version-specific shape of their own: they
//! are the caller-facing limits, refusal/incomplete outcomes, and locked
//! source/occurrence records that the current reader and its lowering
//! pipeline both use unchanged.

use quire_canonical::FixedShape;
use serde::{Deserialize, Serialize};

/// Caller-supplied, exact ceilings for a checked-package read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedPackageReadLimits {
    /// Maximum canonical wire bytes.
    ///
    /// There is no depth limit: the closed body grammar fixes the JSON depth
    /// of every package whatever its node count (FR-038-AC-117).
    pub bytes: u64,
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
            nodes: 10_000,
            edges: 100_000,
            occurrences: 100_000,
            diagnostics: 10_000,
            work: 1_000_000,
        }
    }
}

/// A named resource meter in the closed I04 incomplete outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
pub enum CheckedPackageLimit {
    /// Wire byte budget.
    Bytes,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
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
    /// The package, or something it selects, does not match the content
    /// identity it names: the `package_id`, identity preimage or projection,
    /// a selected domain package document, a dependency's `package_id`, or a
    /// lock that contradicts itself.
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
    /// A well-formed relationship end asks for a traversal form its bounds cannot express.
    UnsupportedConstruct,
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
    /// Two nodes carry the same declared `qualified_name`, or a model-owned
    /// member's name resolves to two exposed effective members.
    AmbiguousDeclaration,
    /// The caller supplied no domain package document under a
    /// `model_selections` digest (FR-154 admission, FR-322 "Model-owned
    /// members" step 1).
    MissingImport,
    /// `unknown_profile`: a temporal clause's `temporal_profile` law names no
    /// member of QSpec FR-250's Values table (QSpec FR-272). Paired with
    /// [`CheckedPackageRefusalCause::UnsupportedSelection`] or
    /// [`CheckedPackageRefusalCause::WrongSelectionRole`].
    UnknownProfile,
}

/// Stable machine cause paired with a [`CheckedPackageRefusalCode`] under
/// FR-322's closed `DiagnosticCausePairing` (`schema.json`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "kebab-case")]
pub enum CheckedPackageRefusalCause {
    /// Malformed JSON document.
    MalformedJson,
    /// A required member is missing.
    MissingMember,
    /// An unknown member is present.
    UnknownMember,
    /// A member has the wrong JSON kind.
    WrongValueKind,
    /// Selected definitions form a cycle.
    DefinitionCycle,
    /// Selected definitions disagree.
    IncompatibleDefinition,
    /// Selected feature sets disagree.
    FeatureSetMismatch,
    /// A resource charge could not be made.
    InsufficientNextCharge,
    /// Selected content has a different recomputed identity.
    ContentMismatch,
    /// `missing-name`: an entry naming no node of the graph, or a node that
    /// is not a declared dependency of the frame node that names it.
    MissingName,
    /// `malformed-declaration`: an entry naming a declared dependency whose
    /// tag and semantic form its member does not admit.
    MalformedDeclaration,
    /// `wrong-export`: a protocol `related.relationship` target is not a
    /// `relation`/`relationship` declaration of the selected model.
    WrongExport,
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
    /// `expression-form`: a relationship destination has an unbounded or ordered multiplicity.
    ExpressionForm,
    /// `declaration-form`: a protocol control's selected profile cannot admit
    /// the awaited event or a repeat body with a path that makes no step.
    DeclarationForm,
    /// `declaration-nominal-mismatch`: a node's declared `qualified_name`
    /// differs from its nominal identity preimage's `qualified_declaration`.
    DeclarationNominalMismatch,
    /// `ambiguous-name`: another node carries the same declared
    /// `qualified_name`, or a model-owned member's name matches two exposed
    /// effective members of its declaring type.
    AmbiguousName,
    /// `missing-selection`: no document is supplied under a selected domain
    /// package digest, or a model declaration node's owner is not a
    /// lock-selected domain package declaration.
    MissingSelection,
    /// `digest-domain-mismatch`: a domain package selection's digest domain
    /// is not `sha256-jcs`.
    DigestDomainMismatch,
    /// Unreadable selected model bytes have a different raw SHA-256 digest.
    ByteDigestMismatch,
    /// `wrong-model-selection`: a supplied domain package document's own
    /// identity differs from its selection's.
    WrongModelSelection,
    /// `conflicting-binding`: two declarations of one domain package share
    /// one IR node identity.
    ConflictingBinding,
    /// `unpreserved-model-meaning`: a domain package multiplicity has
    /// `lower > upper`.
    UnpreservedModelMeaning,
    /// `conflicting-definition`: two `dependency_selections` entries name one
    /// library identity.
    ConflictingDefinition,
    /// `invalid-value`: a `dependency_selections` entry's identity is not
    /// strictly after its predecessor's in UTF-8 byte order.
    InvalidValue,
    /// `duplicate-member`: a `composite_type`/`union` node whose body names
    /// one member twice (QSpec FR-440 reader join 1); paired with
    /// `invalid_package`. The wire-level repeated object member is the code
    /// [`CheckedPackageRefusalCode::DuplicateMember`], not this cause.
    DuplicateMember,
    /// `type-mismatch`: a `value`/`union_value` node that is no construction
    /// of a member of its union type (QSpec FR-440 reader join 2); paired with
    /// `ill_typed`.
    TypeMismatch,
    /// `non-boolean-root`: a hybrid invariant, guard or initial condition is not Boolean.
    NonBooleanRoot,
    /// `missing-timed-model`: timed clauses have no root.
    MissingTimedModel,
    /// `timed-node-reference`: unresolved or wrong-form timed target.
    TimedNodeReference,
    /// `duplicate-clock-place`: a model repeats a clock place.
    DuplicateClockPlace,
    /// `malformed-clock-place`: a clock path is not canonical.
    MalformedClockPlace,
    /// `invalid-time-quantity`: a clock quantity is not exact nonnegative time.
    InvalidTimeQuantity,
    /// `clock-without-dense-time`: a tick time source owns a clock place.
    ClockWithoutDenseTime,
    /// `unsupported-selection`: a clause's `temporal_profile` law names an
    /// identity that is no member of QSpec FR-250's Values table and that the
    /// package's own lock selects under no other role; paired with
    /// `unknown_profile`.
    UnsupportedSelection,
    /// `wrong-selection-role`: a clause's `temporal_profile` law names an
    /// identity the package's own lock selects under a role other than
    /// `temporal_profile`; paired with `unknown_profile`.
    WrongSelectionRole,
    /// `inexact-integer`: a selected model document holds a number whose text
    /// denotes a whole value past 2^53, however spelled; paired with
    /// `noncanonical_wire` (QSpec FR-272).
    InexactInteger,
    /// `inexact-number`: a selected model document holds a number whose exact
    /// value its RFC 8785 encoding loses; paired with `noncanonical_wire`
    /// (QSpec FR-272).
    InexactNumber,
}

/// An RFC 6901 JSON pointer into the checked-package document the reader
/// was given.
///
/// The reader builds every pointer from its own traversal position, with
/// concrete array indices and `~0`/`~1` escaping, and every pointer it
/// returns resolves in that document: a refusal about a member the document
/// lacks points at the object that lacks it. The empty pointer names the whole
/// document.
#[derive(Clone, Debug, Default, Eq, FixedShape, Hash, Ord, PartialEq, PartialOrd)]
pub struct JsonPointer(Box<str>);

impl Serialize for JsonPointer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for JsonPointer {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = Box::<str>::deserialize(deserializer)?;
        Self::parse(&text).ok_or_else(|| serde::de::Error::custom("invalid JSON pointer"))
    }
}

impl JsonPointer {
    /// The pointer to the whole document.
    pub fn root() -> Self {
        Self::default()
    }

    /// Parses an RFC 6901 pointer: empty, or `/`-prefixed reference tokens in
    /// which every `~` begins `~0` or `~1`.
    // Parses the RFC 6901 pointer text of a refusal.
    pub fn parse(pointer: &str) -> Option<Self> {
        if !pointer.is_empty() && !pointer.starts_with('/') {
            return None;
        }
        let mut bytes = pointer.bytes();
        while let Some(byte) = bytes.next() {
            if byte == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
                return None;
            }
        }
        Some(Self(pointer.into()))
    }

    /// Wraps text the reader assembled with [`push_key`]/[`push_index`].
    pub(super) fn from_escaped(text: String) -> Self {
        Self(text.into())
    }

    /// The pointer text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// This pointer extended by one object member name, escaped per RFC 6901.
    pub fn key(self, key: &str) -> Self {
        let mut text = String::from(self.0);
        push_key(&mut text, key);
        Self(text.into())
    }

    /// This pointer extended by one array index.
    pub fn index(self, index: usize) -> Self {
        let mut text = String::from(self.0);
        push_index(&mut text, index);
        Self(text.into())
    }
}

pub(super) fn push_key(text: &mut String, key: &str) {
    text.push('/');
    for character in key.chars() {
        match character {
            '~' => text.push_str("~0"),
            '/' => text.push_str("~1"),
            other => text.push(other),
        }
    }
}

pub(super) fn push_index(text: &mut String, index: usize) {
    use std::fmt::Write as _;
    // Writing to a `String` cannot fail.
    let _ = write!(text, "/{index}");
}

impl std::fmt::Display for JsonPointer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl AsRef<str> for JsonPointer {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Clause information carried by an extension reader refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedExtensionRefusalDetail {
    /// The clause or first absent member.
    pub member: Box<str>,
    /// The required type, when a clause type differs.
    pub expected_type: Option<Box<str>>,
    /// The encountered type, when a clause type differs.
    pub actual_type: Option<Box<str>>,
    /// The referenced subnode's exact semantic type identity at a mismatch.
    pub actual_semantic_type: Option<CheckedNodeId>,
}

/// The two identities involved in a terminal declaration duplicate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedTerminalDuplicate {
    /// Distinct terminal nodes for the same model, in node-id order.
    Nodes {
        /// The earlier terminal node.
        first: CheckedNodeId,
        /// The terminal node whose uniqueness check failed.
        second: CheckedNodeId,
    },
    /// Repeated claim occurrences on one terminal node.
    Occurrences {
        /// The terminal node carrying both occurrences.
        node: CheckedNodeId,
        /// Its first claim occurrence.
        first: CheckedOccurrence,
        /// Its second claim occurrence.
        second: CheckedOccurrence,
    },
}

/// Protocol control information retained at a typed reader refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedProtocolRefusalDetail {
    /// A protocol expression reads a binder outside its stage's readable set.
    BinderScope {
        /// The binder's declared name.
        name: Box<str>,
        /// The slot stage.
        stage: Box<str>,
        /// The clause or compensation template owning the slot.
        owner: CheckedNodeId,
        /// Whether the read is forward, foreign or unavailable at this stage.
        read: Box<str>,
    },
    /// A binders-only slot reaches model state.
    ModelStateRead {
        /// The slot that bars model state.
        slot: Box<str>,
    },
    /// Controls participating in a derived causal cycle, in control-tree order.
    CausalCycle {
        /// The controls on the cycle.
        controls: Vec<CheckedNodeId>,
    },
    /// A repeat body has a path that does not take a step.
    RepeatProgress {
        /// The offending repeat control.
        control: CheckedNodeId,
        /// The lock-selected protocol profile.
        profile: CheckedArtifactRef,
    },
    /// A payload binder differs from the channel's carried type.
    PayloadType {
        /// The channel's carried type.
        expected: CheckedNodeId,
        /// The binder's declared type.
        actual: CheckedNodeId,
    },
    /// A protocol relationship end term does not conform to its selected end.
    RelatedEndType {
        /// The relationship whose end was checked.
        relationship: CheckedNodeId,
        /// The offending end, `from` or `to`.
        end: Box<str>,
        /// The selected relationship end's object type declaration.
        expected: CheckedNodeId,
        /// The end term's semantic type, when it has one.
        actual: Option<CheckedNodeId>,
    },
    /// An awaited control cannot produce the event shape the profile requires.
    AwaitedEvent {
        /// The offending control operator.
        form: Box<str>,
        /// The lock-selected protocol profile.
        profile: CheckedArtifactRef,
    },
    /// A commit is unconditionally taken before its paired forward effect.
    CommitPosition {
        /// The lock-selected protocol profile.
        profile: CheckedArtifactRef,
        /// The offending template member.
        member: Box<str>,
        /// The commit control.
        commit: CheckedNodeId,
        /// The forward effect control.
        effect: CheckedNodeId,
        /// The stable reason for rejecting this placement.
        reason: Box<str>,
    },
    /// A control interval has its bounds in reverse order.
    Interval {
        /// The argument holding the interval.
        member: Box<str>,
        /// The required order of the bounds.
        expected: Box<str>,
        /// The interval bounds as written.
        actual: Box<str>,
    },
}

/// A typed refusal with a stable code and the location of the value it is
/// about.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackageRefusal {
    /// The condition callers can distinguish without parsing prose.
    pub code: CheckedPackageRefusalCode,
    /// The RFC 6901 pointer of the value the refusal is about; absent when
    /// the refusal concerns the byte stream rather than a value (malformed
    /// JSON, non-canonical bytes).
    pub path: Option<JsonPointer>,
    /// The other importer-rooted selection path in a dependency diamond conflict.
    pub related_path: Option<JsonPointer>,
    /// The cause tag paired with `code` under FR-322's `DiagnosticCausePairing`,
    /// present exactly when this reader determined one.
    pub cause: Option<CheckedPackageRefusalCause>,
    /// The node key of the offending entry or node, present whenever this
    /// reader located the refusal at a specific graph node.
    pub locus: Option<CheckedNodeId>,
    /// The `contract_version` string the reader read, present exactly when
    /// `code` is [`CheckedPackageRefusalCode::UnknownContractVersion`].
    pub contract_version: Option<Box<str>>,
    /// The RFC 6901 pointer supplied by a requirement in addition to `path`:
    /// into a selected model document for its noncanonical number, or into
    /// the checked package for an unheld extension definition (FR-469).
    pub document_pointer: Option<JsonPointer>,
    /// Selected or pinned identity in a content or byte digest mismatch.
    pub expected: Option<CheckedSemanticId>,
    /// Recomputed identity in a content or byte digest mismatch.
    pub actual: Option<CheckedSemanticId>,
    /// Extension clause information at a missing-member or type-mismatch refusal.
    pub extension_detail: Option<Box<CheckedExtensionRefusalDetail>>,
    /// Protocol event, type or interval context for a control refusal.
    pub protocol_detail: Option<Box<CheckedProtocolRefusalDetail>>,
    /// Both parties to a terminal declaration duplicate, when applicable.
    pub terminal_duplicate: Option<CheckedTerminalDuplicate>,
    /// The key already derived at a stale-node-key refusal site, when one
    /// exists. It is meaningful only with the original reader-returned locus.
    expected_node_id: Option<Box<CheckedNodeId>>,
}

/// Named fields for constructing a refusal inside this crate. The expected
/// key is set only by the dedicated stale-key constructor.
pub(crate) struct RefusalFields {
    pub(crate) code: CheckedPackageRefusalCode,
    pub(crate) path: Option<JsonPointer>,
    pub(crate) cause: Option<CheckedPackageRefusalCause>,
    pub(crate) locus: Option<CheckedNodeId>,
    pub(crate) contract_version: Option<Box<str>>,
    pub(crate) document_pointer: Option<JsonPointer>,
}

impl CheckedPackageRefusal {
    pub(crate) fn new(fields: RefusalFields) -> Self {
        Self {
            code: fields.code,
            path: fields.path,
            related_path: None,
            cause: fields.cause,
            locus: fields.locus,
            contract_version: fields.contract_version,
            document_pointer: fields.document_pointer,
            expected: None,
            actual: None,
            extension_detail: None,
            protocol_detail: None,
            terminal_duplicate: None,
            expected_node_id: None,
        }
    }

    /// A selected or pinned identity differs from the supplied content.
    pub(crate) fn mismatch(
        path: JsonPointer,
        cause: CheckedPackageRefusalCause,
        expected: CheckedSemanticId,
        actual: CheckedSemanticId,
    ) -> Self {
        debug_assert!(matches!(
            cause,
            CheckedPackageRefusalCause::ContentMismatch
                | CheckedPackageRefusalCause::ByteDigestMismatch
        ));
        let mut refusal = Self::new(RefusalFields {
            code: CheckedPackageRefusalCode::StaleDependency,
            path: Some(path),
            cause: Some(cause),
            locus: None,
            contract_version: None,
            document_pointer: None,
        });
        refusal.expected = Some(expected);
        refusal.actual = Some(actual);
        refusal
    }

    pub(crate) fn stale_node_key(
        path: JsonPointer,
        locus: CheckedNodeId,
        expected_node_id: CheckedNodeId,
    ) -> Self {
        let mut refusal = Self::new(RefusalFields {
            code: CheckedPackageRefusalCode::InvalidPackage,
            path: Some(path),
            cause: Some(CheckedPackageRefusalCause::StaleNodeKey),
            locus: Some(locus),
            contract_version: None,
            document_pointer: None,
        });
        refusal.expected_node_id = Some(Box::new(expected_node_id));
        refusal
    }

    /// The production-derived node key retained with a stale-key refusal, if
    /// the reader could derive one. The original [`Self::locus`] identifies the
    /// stored key that refused.
    pub fn expected_node_id(&self) -> Option<&CheckedNodeId> {
        self.expected_node_id.as_deref()
    }
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
    /// The RFC 6901 pointer of the value whose charge failed; present for
    /// every limit except [`CheckedPackageLimit::Bytes`], which is charged
    /// before any value is parsed.
    pub path: Option<JsonPointer>,
}

/// The identity of one locked artifact, used to group a source map's regions
/// by the source they fall in.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CheckedArtifactLocator {
    /// Artifact identity authority.
    pub authority: Box<str>,
    /// Artifact identity.
    pub identity: Box<str>,
    /// Identity domain of the bytes.
    pub domain: Box<str>,
}

/// A semantic digest with an explicit domain and algorithm.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Serialize, Deserialize, FixedShape)]
#[cfg_attr(not(test), derive(Eq, PartialEq, Ord, PartialOrd))]
#[cfg_attr(test, derive(Eq))]
#[serde(deny_unknown_fields)]
pub struct CheckedNodeId {
    /// Must be `quire.checked-semantic-node/v1`.
    pub domain: Box<str>,
    /// Lowercase SHA-256 digest.
    pub digest: Box<str>,
}

#[cfg(test)]
impl PartialEq for CheckedNodeId {
    fn eq(&self, other: &Self) -> bool {
        checked_node_comparisons::visited();
        self.domain == other.domain && self.digest == other.digest
    }
}

#[cfg(test)]
impl PartialOrd for CheckedNodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl Ord for CheckedNodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        checked_node_comparisons::visited();
        (&self.domain, &self.digest).cmp(&(&other.domain, &other.digest))
    }
}

#[cfg(test)]
pub(crate) mod checked_node_comparisons {
    use std::cell::Cell;

    thread_local! {
        static COUNT: Cell<u64> = const { Cell::new(0) };
    }

    pub(crate) fn visited() {
        COUNT.with(|count| count.set(count.get().saturating_add(1)));
    }

    pub(crate) fn reset() {
        COUNT.with(|count| count.set(0));
    }

    pub(crate) fn count() -> u64 {
        COUNT.with(Cell::get)
    }
}

/// A source occurrence role.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedOccurrence {
    /// Semantic role.
    pub role: CheckedOccurrenceRole,
    /// Stable ordinal for repeated roles.
    pub ordinal: u64,
}

/// A definition named by its identity alone: QSpec's `DefinitionRef`, the closed
/// shape `{authority, identity}`. Two references are equal when both members
/// are equal; the reader compares nothing else about a definition.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedArtifactRef {
    /// Definition authority.
    pub authority: Box<str>,
    /// Definition identity.
    pub identity: Box<str>,
}

/// A raw source document bound to the exact bytes the producer read: QSpec's
/// `RawSourceRef`, the closed shape `{authority, identity, digest_domain,
/// digest}`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedSourceRef {
    /// Source authority.
    pub authority: Box<str>,
    /// Source identity.
    pub identity: Box<str>,
    /// Typed byte-digest domain, `quire.source.bytes/v1`.
    pub digest_domain: Box<str>,
    /// Lowercase SHA-256 digest of the source bytes.
    pub digest: Box<str>,
}

/// A selected definition-role identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedSelection {
    /// The closed selection role.
    pub role: super::v2::CheckedSelectionRole,
    /// Selected definition artifact.
    pub definition: CheckedArtifactRef,
}

/// A source interval under an exact raw-source identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedSourceRegion {
    /// Raw source document identity.
    pub source: CheckedSourceRef,
    /// Inclusive byte start.
    pub start: u64,
    /// Exclusive byte end.
    pub end: u64,
}

/// Exact source correspondence for one node occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedCapability {
    /// Capability identifier.
    pub feature: Box<str>,
    /// Producer/model disposition.
    pub disposition: super::v2::CheckedCapabilityDisposition,
}
