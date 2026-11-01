//! Version-neutral I04 `CheckedPackage` vocabulary.
//!
//! These types carry no contract-version-specific shape of their own: they
//! are the caller-facing limits, refusal/incomplete outcomes, and locked
//! source/occurrence records that the current reader and its lowering
//! pipeline both use unchanged.

use quire_canonical::{Encode, Error as CanonicalError, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};

/// Caller-supplied, exact ceilings for a checked-package read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedPackageReadLimits {
    /// Maximum canonical wire bytes.
    ///
    /// There is no fixed depth limit. Structural node paths and canonical
    /// encoding use heap worklists bounded by this byte ceiling.
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
    /// An application node's `operation` failed catalog validation.
    InvalidPackage,
    /// An application node's argument, member or leaf is ill-typed against
    /// its catalogued operation.
    IllTyped,
    /// Two nodes carry the same declared `qualified_name`, or a model-owned
    /// member's name resolves to two exposed effective members.
    AmbiguousDeclaration,
    /// The caller supplied no domain package document under a
    /// selected domain package identity (FR-154 admission, FR-322 "Model-owned
    /// members" step 1).
    MissingImport,
    /// `unknown_profile`: a temporal clause's `temporal_profile` law names no
    /// member of QSpec FR-250's Values table (QSpec FR-272). Paired with
    /// [`CheckedPackageRefusalCause::UnsupportedSelection`] or
    /// [`CheckedPackageRefusalCause::WrongSelectionRole`].
    UnknownProfile,
    /// A decision reads a fact unavailable in its owner's observation.
    WrongSnapshot,
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
    /// `missing-name`: an entry naming no node of the graph, or a node that
    /// is not a declared dependency of the frame node that names it.
    MissingName,
    /// A decision reads a fact of another role, an ineligible kind, or an unlisted fact.
    WrongObservation,
    /// `malformed-declaration`: an entry naming a declared dependency whose
    /// tag and semantic form its member does not admit.
    MalformedDeclaration,
    /// `wrong-export`: a protocol `related.relationship` target is not a
    /// `relation`/`relationship` declaration of the selected model.
    WrongExport,
    /// `invalid-derived-shape`: a correctly keyed derived node has an
    /// inadmissible semantic target.
    InvalidDerivedShape,
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
    /// differs from its declaration's `qualified_name` path.
    DeclarationNominalMismatch,
    /// `ambiguous-name`: another node carries the same declared
    /// `qualified_name`, or a model-owned member's name matches two exposed
    /// effective members of its declaring type.
    AmbiguousName,
    /// `missing-selection`: no document is supplied under a selected domain
    /// package identity, or a model declaration node's owner names no
    /// `model_root`.
    MissingSelection,
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
    /// A binders-only slot reaches model state, or a decision reads a bare record binder.
    ModelStateRead {
        /// The slot that bars model state.
        slot: Box<str>,
    },
    /// A choice or repeat decision reads a fact outside its visible observation.
    DecisionFact {
        /// The field-read expression node.
        node: CheckedNodeId,
        /// The role making the decision.
        owner: CheckedNodeId,
        /// The record binder and field read by the expression.
        read: CheckedProtocolDecisionRead,
        /// `other-role`, `ineligible-kind`, or `unlisted`.
        reason: Box<str>,
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

/// The record binder and field named by a decision-fact read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CheckedProtocolDecisionRead {
    /// The record binder node.
    pub binder: CheckedNodeId,
    /// The field selected from that binder.
    pub field: Box<str>,
}

/// One exact location of a typed refusal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CheckedPackageRefusalLocation {
    /// RFC 6901 pointer of the value; absent only for a byte-stream defect.
    pub path: Option<JsonPointer>,
    /// Exact admitted source region of this value, when one exists.
    pub source_region: Option<CheckedSourceRegion>,
}

/// A typed refusal with a stable code and one or more ordered locations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CheckedPackageRefusal {
    /// The condition callers can distinguish without parsing prose.
    pub code: CheckedPackageRefusalCode,
    /// Nonempty locations in declaration order; the first is primary.
    pub loci: Vec<CheckedPackageRefusalLocation>,
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
    /// into a selected model document for its noncanonical number or an
    /// unsupported variant payload, or into the checked package for an
    /// unheld extension definition (FR-469).
    pub document_pointer: Option<JsonPointer>,
    /// Extension clause information at a missing-member or type-mismatch refusal.
    pub extension_detail: Option<Box<CheckedExtensionRefusalDetail>>,
    /// Protocol event, type or interval context for a control refusal.
    pub protocol_detail: Option<Box<CheckedProtocolRefusalDetail>>,
    /// Both parties to a terminal declaration duplicate, when applicable.
    pub terminal_duplicate: Option<CheckedTerminalDuplicate>,
}

impl std::fmt::Display for CheckedPackageRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, formatter)
    }
}

/// Named fields for constructing a refusal inside this crate.
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
            loci: vec![CheckedPackageRefusalLocation {
                path: fields.path,
                source_region: None,
            }],
            cause: fields.cause,
            locus: fields.locus,
            contract_version: fields.contract_version,
            document_pointer: fields.document_pointer,
            extension_detail: None,
            protocol_detail: None,
            terminal_duplicate: None,
        }
    }

    /// The primary RFC 6901 pointer, if this is a value refusal.
    pub(crate) fn primary_path(&self) -> Option<&JsonPointer> {
        self.loci
            .first()
            .and_then(|location| location.path.as_ref())
    }

    /// Replace the primary value pointer after a more precise decode locates it.
    pub(crate) fn set_primary_path(&mut self, path: Option<JsonPointer>) {
        self.loci[0].path = path;
    }

    /// Add another independently identified refusal location in declaration order.
    pub(crate) fn push_path(&mut self, path: JsonPointer) {
        self.loci.push(CheckedPackageRefusalLocation {
            path: Some(path),
            source_region: None,
        });
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

/// The nominal identity of one source artifact.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CheckedArtifactLocator {
    /// Artifact identity authority.
    pub authority: Box<str>,
    /// Artifact identity.
    pub identity: Box<str>,
}

/// One segment of a checked node's structural path.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CheckedNodePathSegment {
    /// An authored or derived path spelling.
    Name(Box<str>),
    /// An operand node's structural identity.
    Node(std::sync::Arc<CheckedNodeId>),
    /// A graph-local operand index on the V2 wire, before key resolution.
    Index(u64),
}

impl Serialize for CheckedNodePathSegment {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Self::Name(name) => serializer.serialize_str(name),
            Self::Index(index) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("node", index)?;
                map.end()
            }
            Self::Node(node) => node.serialize(serializer),
        }
    }
}

/// A checked semantic graph node identity.
#[derive(Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedNodeId {
    /// The name of the owning package.
    pub package: Box<str>,
    /// Nonempty declared or derived path.
    pub path: Vec<CheckedNodePathSegment>,
    /// Position among nodes sharing the package and path.
    pub ordinal: u64,
}

/// One iterative Debug view of roots and their shared structural children.
pub(crate) struct CheckedNodeTableDebug<'a> {
    roots: Vec<&'a CheckedNodeId>,
}

impl<'a> CheckedNodeTableDebug<'a> {
    pub(crate) fn new(roots: impl IntoIterator<Item = &'a CheckedNodeId>) -> Self {
        Self {
            roots: roots.into_iter().collect(),
        }
    }
}

impl std::fmt::Debug for CheckedNodeTableDebug<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::collections::HashMap;

        let mut nodes = Vec::new();
        let mut positions = HashMap::<*const CheckedNodeId, usize>::new();
        for root in &self.roots {
            let pointer = std::ptr::from_ref(*root);
            if let std::collections::hash_map::Entry::Vacant(entry) = positions.entry(pointer) {
                entry.insert(nodes.len());
                nodes.push(*root);
            }
        }
        let mut at = 0;
        while at < nodes.len() {
            for segment in &nodes[at].path {
                if let CheckedNodePathSegment::Node(child) = segment {
                    let pointer = std::sync::Arc::as_ptr(child);
                    if let std::collections::hash_map::Entry::Vacant(entry) =
                        positions.entry(pointer)
                    {
                        entry.insert(nodes.len());
                        nodes.push(child);
                    }
                }
            }
            at += 1;
        }

        formatter.write_str("NodeTable { roots: [")?;
        for (at, root) in self.roots.iter().enumerate() {
            if at != 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "#{}", positions[&std::ptr::from_ref(*root)])?;
        }
        formatter.write_str("], nodes: [")?;
        for (at, node) in nodes.iter().enumerate() {
            if at != 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "#{at} {{ package: {:?}, path: [", node.package)?;
            for (part, segment) in node.path.iter().enumerate() {
                if part != 0 {
                    formatter.write_str(", ")?;
                }
                match segment {
                    CheckedNodePathSegment::Name(name) => write!(formatter, "Name({name:?})")?,
                    CheckedNodePathSegment::Index(index) => write!(formatter, "Index({index})")?,
                    CheckedNodePathSegment::Node(child) => write!(
                        formatter,
                        "Node(#{})",
                        positions[&std::sync::Arc::as_ptr(child)]
                    )?,
                }
            }
            write!(formatter, "], ordinal: {} }}", node.ordinal)?;
        }
        formatter.write_str("] }")
    }
}

impl std::fmt::Debug for CheckedNodeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("CheckedNodeId")
            .field(&CheckedNodeTableDebug::new(std::iter::once(self)))
            .finish()
    }
}

impl<'de> Deserialize<'de> for CheckedNodeId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{Error as _, MapAccess, Visitor};

        // Wire paths are flat: an operand is an index, never a nested ID.
        // Decode through serde's data model so buffered enum content and
        // borrowed JSON values work as well as the text deserializer.
        struct Segment(CheckedNodePathSegment);
        struct SegmentVisitor;
        impl<'de> Visitor<'de> for SegmentVisitor {
            type Value = Segment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a path string or an object containing only a node index")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Segment, E> {
                Ok(Segment(CheckedNodePathSegment::Name(value.into())))
            }

            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Segment, E> {
                Ok(Segment(CheckedNodePathSegment::Name(value.into())))
            }

            #[ix_trace_rs::string_edge]
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Segment, A::Error> {
                let name: String = map
                    .next_key()?
                    .ok_or_else(|| A::Error::missing_field("node"))?;
                if name != "node" {
                    return Err(A::Error::unknown_field(&name, &["node"]));
                }
                let index: u64 = map.next_value()?;
                if index > 9_007_199_254_740_991 {
                    return Err(A::Error::custom("node index exceeds I-JSON bound"));
                }
                if let Some(name) = map.next_key::<String>()? {
                    return Err(if name == "node" {
                        A::Error::duplicate_field("node")
                    } else {
                        A::Error::unknown_field(&name, &["node"])
                    });
                }
                Ok(Segment(CheckedNodePathSegment::Index(index)))
            }
        }
        impl<'de> Deserialize<'de> for Segment {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserializer.deserialize_any(SegmentVisitor)
            }
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireNodeId {
            package: Box<str>,
            path: Vec<Segment>,
            ordinal: u64,
        }
        struct NodeVisitor;
        impl<'de> Visitor<'de> for NodeVisitor {
            type Value = WireNodeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a flat node ID object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<WireNodeId, A::Error> {
                WireNodeId::deserialize(serde::de::value::MapAccessDeserializer::new(map))
            }
        }
        let wire = deserializer.deserialize_map(NodeVisitor)?;
        Ok(Self {
            package: wire.package,
            path: wire.path.into_iter().map(|segment| segment.0).collect(),
            ordinal: wire.ordinal,
        })
    }
}

impl CheckedNodeId {
    /// Read a V2 node ID already held as JSON without accepting nested IDs
    /// in its path. The supplied-value boundary also validates the root's
    /// nonempty package/path and exact ordinal bounds.
    pub(crate) fn from_value(value: &serde_json::Value) -> Option<Self> {
        let node = Self::deserialize(value).ok()?;
        node.is_well_formed().then_some(node)
    }

    /// Whether every segment and nested operand is within the V2 wire bounds.
    pub(crate) fn is_well_formed(&self) -> bool {
        let mut pending = vec![self];
        while let Some(candidate) = pending.pop() {
            if candidate.package.is_empty()
                || candidate.path.is_empty()
                || candidate.ordinal > 9_007_199_254_740_991
            {
                return false;
            }
            for segment in &candidate.path {
                match segment {
                    CheckedNodePathSegment::Name(_) => {}
                    CheckedNodePathSegment::Index(index) if *index <= 9_007_199_254_740_991 => {}
                    CheckedNodePathSegment::Index(_) => return false,
                    CheckedNodePathSegment::Node(node) => pending.push(node),
                }
            }
        }
        true
    }
}

impl Drop for CheckedNodeId {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.path);
        while let Some(segment) = pending.pop() {
            if let CheckedNodePathSegment::Node(node) = segment {
                if let Ok(mut node) = std::sync::Arc::try_unwrap(node) {
                    pending.append(&mut node.path);
                }
            }
        }
    }
}

impl Clone for CheckedNodeId {
    fn clone(&self) -> Self {
        Self {
            package: self.package.clone(),
            path: self.path.clone(),
            ordinal: self.ordinal,
        }
    }
}

impl PartialEq for CheckedNodeId {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for CheckedNodeId {}

impl PartialOrd for CheckedNodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CheckedNodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        enum Task<'a> {
            Node(&'a CheckedNodeId, &'a CheckedNodeId),
            Path(&'a [CheckedNodePathSegment], &'a [CheckedNodePathSegment]),
            Ordinal(u64, u64),
        }
        let mut tasks = vec![Task::Node(self, other)];
        while let Some(task) = tasks.pop() {
            match task {
                Task::Node(left, right) => {
                    #[cfg(test)]
                    checked_node_comparisons::visited();
                    if std::ptr::eq(left, right) {
                        continue;
                    }
                    match left.package.cmp(&right.package) {
                        Ordering::Equal => {
                            tasks.push(Task::Ordinal(left.ordinal, right.ordinal));
                            tasks.push(Task::Path(&left.path, &right.path));
                        }
                        order => return order,
                    }
                }
                Task::Path(left, right) => match (left.split_first(), right.split_first()) {
                    (None, None) => {}
                    (None, Some(_)) => return Ordering::Less,
                    (Some(_), None) => return Ordering::Greater,
                    (Some((head_left, rest_left)), Some((head_right, rest_right))) => {
                        tasks.push(Task::Path(rest_left, rest_right));
                        match (head_left, head_right) {
                            (CheckedNodePathSegment::Name(a), CheckedNodePathSegment::Name(b)) => {
                                match a.cmp(b) {
                                    Ordering::Equal => {}
                                    order => return order,
                                }
                            }
                            (CheckedNodePathSegment::Name(_), CheckedNodePathSegment::Node(_)) => {
                                return Ordering::Less
                            }
                            (CheckedNodePathSegment::Name(_), CheckedNodePathSegment::Index(_)) => {
                                return Ordering::Less
                            }
                            (CheckedNodePathSegment::Index(_), CheckedNodePathSegment::Name(_)) => {
                                return Ordering::Greater
                            }
                            (
                                CheckedNodePathSegment::Index(a),
                                CheckedNodePathSegment::Index(b),
                            ) => match a.cmp(b) {
                                Ordering::Equal => {}
                                order => return order,
                            },
                            (CheckedNodePathSegment::Index(_), CheckedNodePathSegment::Node(_)) => {
                                return Ordering::Less
                            }
                            (CheckedNodePathSegment::Node(_), CheckedNodePathSegment::Index(_)) => {
                                return Ordering::Greater
                            }
                            (CheckedNodePathSegment::Node(_), CheckedNodePathSegment::Name(_)) => {
                                return Ordering::Greater
                            }
                            (CheckedNodePathSegment::Node(a), CheckedNodePathSegment::Node(b)) => {
                                if !std::sync::Arc::ptr_eq(a, b) {
                                    tasks.push(Task::Node(a, b));
                                }
                            }
                        }
                    }
                },
                Task::Ordinal(left, right) => match left.cmp(&right) {
                    Ordering::Equal => {}
                    order => return order,
                },
            }
        }
        Ordering::Equal
    }
}

/// Rank resolved structural IDs in `CheckedNodeId::cmp` order without
/// repeatedly walking a shared operand spine. Children enter the order tree
/// before parents, so a parent comparison reads the positions of its children.
pub(crate) fn rank_checked_node_ids(keys: &[std::sync::Arc<CheckedNodeId>]) -> Vec<usize> {
    use std::collections::HashMap;

    struct Entry<'a> {
        key: &'a CheckedNodeId,
        parent: Option<usize>,
        left: Option<usize>,
        right: Option<usize>,
        size: usize,
        priority: u64,
    }

    struct Order<'a> {
        entries: Vec<Entry<'a>>,
        by_pointer: HashMap<*const CheckedNodeId, usize>,
        root: Option<usize>,
        random: u64,
    }

    impl<'a> Order<'a> {
        fn size(&self, index: Option<usize>) -> usize {
            index.map_or(0, |at| self.entries[at].size)
        }

        fn position(&self, mut at: usize) -> usize {
            let mut position = self.size(self.entries[at].left);
            while let Some(parent) = self.entries[at].parent {
                if self.entries[parent].right == Some(at) {
                    position += self.size(self.entries[parent].left) + 1;
                }
                at = parent;
            }
            position
        }

        fn compare(&self, left: &CheckedNodeId, right: &CheckedNodeId) -> std::cmp::Ordering {
            use std::cmp::Ordering;
            let package = left.package.cmp(&right.package);
            if package != Ordering::Equal {
                return package;
            }
            for (a, b) in left.path.iter().zip(&right.path) {
                let order = match (a, b) {
                    (CheckedNodePathSegment::Name(a), CheckedNodePathSegment::Name(b)) => a.cmp(b),
                    (CheckedNodePathSegment::Index(a), CheckedNodePathSegment::Index(b)) => {
                        a.cmp(b)
                    }
                    (CheckedNodePathSegment::Node(a), CheckedNodePathSegment::Node(b)) => {
                        let a = self.by_pointer[&std::sync::Arc::as_ptr(a)];
                        let b = self.by_pointer[&std::sync::Arc::as_ptr(b)];
                        self.position(a).cmp(&self.position(b))
                    }
                    (CheckedNodePathSegment::Name(_), _) => Ordering::Less,
                    (_, CheckedNodePathSegment::Name(_)) => Ordering::Greater,
                    (CheckedNodePathSegment::Index(_), _) => Ordering::Less,
                    (CheckedNodePathSegment::Node(_), _) => Ordering::Greater,
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
            left.path
                .len()
                .cmp(&right.path.len())
                .then_with(|| left.ordinal.cmp(&right.ordinal))
        }

        fn rotate_up(&mut self, child: usize) {
            let parent = self.entries[child].parent.expect("rotation has a parent");
            let grand = self.entries[parent].parent;
            if self.entries[parent].left == Some(child) {
                let moved = self.entries[child].right;
                self.entries[parent].left = moved;
                if let Some(moved) = moved {
                    self.entries[moved].parent = Some(parent);
                }
                self.entries[child].right = Some(parent);
            } else {
                let moved = self.entries[child].left;
                self.entries[parent].right = moved;
                if let Some(moved) = moved {
                    self.entries[moved].parent = Some(parent);
                }
                self.entries[child].left = Some(parent);
            }
            self.entries[parent].parent = Some(child);
            self.entries[child].parent = grand;
            if let Some(grand) = grand {
                if self.entries[grand].left == Some(parent) {
                    self.entries[grand].left = Some(child);
                } else {
                    self.entries[grand].right = Some(child);
                }
            } else {
                self.root = Some(child);
            }
            self.entries[parent].size =
                1 + self.size(self.entries[parent].left) + self.size(self.entries[parent].right);
            self.entries[child].size =
                1 + self.size(self.entries[child].left) + self.size(self.entries[child].right);
        }

        fn insert(&mut self, key: &'a CheckedNodeId) -> usize {
            use std::cmp::Ordering;
            let mut parent = None;
            let mut at = self.root;
            let mut side = Ordering::Equal;
            while let Some(current) = at {
                side = self.compare(key, self.entries[current].key);
                if side == Ordering::Equal {
                    return current;
                }
                parent = Some(current);
                at = if side == Ordering::Less {
                    self.entries[current].left
                } else {
                    self.entries[current].right
                };
            }
            // The priority is for tree balance only; it never identifies a
            // package, file or tool, and it does not affect structural order.
            self.random = self
                .random
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let index = self.entries.len();
            self.entries.push(Entry {
                key,
                parent,
                left: None,
                right: None,
                size: 1,
                priority: self.random,
            });
            if let Some(parent) = parent {
                if side == Ordering::Less {
                    self.entries[parent].left = Some(index);
                } else {
                    self.entries[parent].right = Some(index);
                }
                let mut ancestor = Some(parent);
                while let Some(current) = ancestor {
                    self.entries[current].size += 1;
                    ancestor = self.entries[current].parent;
                }
            } else {
                self.root = Some(index);
            }
            while let Some(parent) = self.entries[index].parent {
                if self.entries[index].priority <= self.entries[parent].priority {
                    break;
                }
                self.rotate_up(index);
            }
            index
        }

        fn positions(&self) -> Vec<usize> {
            let mut positions = vec![0; self.entries.len()];
            let mut pending = Vec::new();
            let mut at = self.root;
            let mut next = 0;
            while at.is_some() || !pending.is_empty() {
                while let Some(current) = at {
                    pending.push(current);
                    at = self.entries[current].left;
                }
                let current = pending.pop().expect("nonempty traversal stack");
                positions[current] = next;
                next += 1;
                at = self.entries[current].right;
            }
            positions
        }
    }

    let mut order = Order {
        entries: Vec::new(),
        by_pointer: HashMap::new(),
        root: None,
        random: 0x9e37_79b9_7f4a_7c15,
    };
    for root in keys {
        let mut pending = vec![(root.as_ref(), false)];
        while let Some((key, finish)) = pending.pop() {
            let pointer = std::ptr::from_ref(key);
            if order.by_pointer.contains_key(&pointer) {
                continue;
            }
            if finish {
                let index = order.insert(key);
                order.by_pointer.insert(pointer, index);
            } else {
                pending.push((key, true));
                for segment in key.path.iter().rev() {
                    if let CheckedNodePathSegment::Node(child) = segment {
                        pending.push((child, false));
                    }
                }
            }
        }
    }
    let positions = order.positions();
    keys.iter()
        .map(|key| positions[order.by_pointer[&std::sync::Arc::as_ptr(key)]])
        .collect()
}

impl Encode for CheckedNodeId {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        enum Event<'a> {
            Node(&'a CheckedNodeId),
            Segment(&'a CheckedNodePathSegment),
            EndNode(u64),
        }
        let mut pending = vec![Event::Node(self)];
        while let Some(event) = pending.pop() {
            match event {
                Event::Node(node) => {
                    writer.begin_object()?;
                    writer.name("package")?;
                    writer.string(&node.package)?;
                    writer.name("path")?;
                    writer.begin_array()?;
                    pending.push(Event::EndNode(node.ordinal));
                    pending.extend(node.path.iter().rev().map(Event::Segment));
                }
                Event::Segment(CheckedNodePathSegment::Name(name)) => writer.string(name)?,
                Event::Segment(CheckedNodePathSegment::Index(index)) => {
                    writer.begin_object()?;
                    writer.name("node")?;
                    writer.integer(i128::from(*index))?;
                    writer.end_object()?;
                }
                Event::Segment(CheckedNodePathSegment::Node(node)) => {
                    pending.push(Event::Node(node));
                }
                Event::EndNode(ordinal) => {
                    writer.end_array()?;
                    writer.name("ordinal")?;
                    writer.integer(i128::from(ordinal))?;
                    writer.end_object()?;
                }
            }
        }
        Ok(())
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

#[cfg(test)]
mod structural_node_identity_tests {
    use super::{
        rank_checked_node_ids, CheckedNodeId, CheckedNodePathSegment, CheckedPackageRefusal,
        CheckedPackageRefusalCode, JsonPointer, RefusalFields,
    };

    /// Trace: FR-038-AC-185, TC-048.
    #[test]
    fn refusal_public_forms_retain_only_the_reported_structural_locus() {
        let pre_graph = CheckedPackageRefusal::new(RefusalFields {
            code: CheckedPackageRefusalCode::MalformedWire,
            path: None,
            cause: None,
            locus: None,
            contract_version: None,
            document_pointer: None,
        });
        let pre_graph_json = serde_json::to_value(&pre_graph).expect("serialize refusal");
        assert!(pre_graph_json["locus"].is_null());
        assert!(pre_graph.to_string().contains("locus: None"));

        let id = CheckedNodeId {
            package: "example".into(),
            path: vec![CheckedNodePathSegment::Name("declaration".into())],
            ordinal: 7,
        };
        let located = CheckedPackageRefusal::new(RefusalFields {
            code: CheckedPackageRefusalCode::InvalidSemanticGraph,
            path: JsonPointer::parse("/semantic_graph/nodes/1/node_id"),
            cause: None,
            locus: Some(id.clone()),
            contract_version: None,
            document_pointer: None,
        });
        let cloned = located.clone();
        assert_eq!(cloned.locus, Some(id.clone()));
        let encoded = serde_json::to_value(&located).expect("serialize located refusal");
        assert_eq!(
            encoded["locus"],
            serde_json::to_value(&id).expect("serialize ID")
        );
        assert_eq!(
            encoded["loci"][0]["path"],
            "/semantic_graph/nodes/1/node_id"
        );
        assert!(located.to_string().contains("declaration"));
        assert!(encoded.get("expected_node_id").is_none());
        assert!(encoded.get("actual_digest").is_none());
    }

    /// Trace: FR-038-AC-377, FR-322-AC-278.
    #[test]
    fn supplied_node_id_value_requires_the_exact_structural_shape() {
        let valid = serde_json::json!({"package":"p","path":["owner",{"node":0}],"ordinal":1});
        let parsed = CheckedNodeId::from_value(&valid).expect("indexed structural ID");
        assert_eq!(serde_json::to_value(&parsed).expect("encode ID"), valid);
        for malformed in [
            serde_json::json!(["p", ["owner"], 0]),
            serde_json::json!("p/owner"),
            serde_json::json!({"package":"p","path":[],"ordinal":0}),
            serde_json::json!({"package":"p","path":["owner"],"ordinal":"0"}),
            serde_json::json!({"package":"p","path":["owner"],"ordinal":0,"version":1}),
            serde_json::json!({"package":"p","path":[{"package":"q","path":[],"ordinal":0}],"ordinal":0}),
            serde_json::json!({"package":"p","path":[{"package":"q","path":["type"],"ordinal":0}],"ordinal":0}),
            serde_json::json!({"package":"p","path":[{"node":0,"extra":1}],"ordinal":0}),
        ] {
            assert!(
                CheckedNodeId::from_value(&malformed).is_none(),
                "{malformed}"
            );
        }
    }

    /// Trace: FR-038-AC-364, FR-038-AC-365.
    #[test]
    fn nested_paths_compare_by_segment_kind_then_ordinal() {
        let named = CheckedNodeId {
            package: "example".into(),
            path: vec![CheckedNodePathSegment::Name("A".into())],
            ordinal: 0,
        };
        let longer = CheckedNodeId {
            package: named.package.clone(),
            path: vec![
                CheckedNodePathSegment::Name("A".into()),
                CheckedNodePathSegment::Name("B".into()),
            ],
            ordinal: named.ordinal,
        };
        let nested = CheckedNodeId {
            package: named.package.clone(),
            path: vec![CheckedNodePathSegment::Node(std::sync::Arc::new(
                named.clone(),
            ))],
            ordinal: named.ordinal,
        };
        assert!(named < longer);
        assert!(longer < nested);
        assert!(
            named
                < CheckedNodeId {
                    package: named.package.clone(),
                    path: named.path.clone(),
                    ordinal: 1
                }
        );
        assert_eq!(
            serde_json::to_value(&nested).expect("serialize structural id"),
            serde_json::json!({"package":"example","path":[
                {"package":"example","path":["A"],"ordinal":0}
            ],"ordinal":0})
        );
        let canonical = quire_canonical::to_vec(&nested, quire_canonical::Limits::new(4096))
            .expect("encode nested structural id");
        let encoded: serde_json::Value =
            serde_json::from_slice(&canonical).expect("decode encoded structural id");
        assert_eq!(
            encoded,
            serde_json::to_value(&nested).expect("serialize id")
        );
    }

    /// Trace: FR-038-AC-364, FR-038-AC-380.
    #[test]
    fn structural_ranks_agree_with_nested_id_order() {
        use std::sync::Arc;

        let mut keys = vec![Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![CheckedNodePathSegment::Name("leaf".into())],
            ordinal: 0,
        })];
        for depth in 0..80 {
            let child = Arc::clone(keys.last().expect("first key exists"));
            keys.push(Arc::new(CheckedNodeId {
                package: if depth % 7 == 0 { "q" } else { "p" }.into(),
                path: vec![
                    CheckedNodePathSegment::Name(if depth % 3 == 0 { "a" } else { "z" }.into()),
                    CheckedNodePathSegment::Node(child),
                ],
                ordinal: (depth % 2) as u64,
            }));
        }
        keys.push(Arc::new(keys[17].as_ref().clone()));
        keys.push(Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![CheckedNodePathSegment::Index(17)],
            ordinal: 0,
        }));
        keys.reverse();
        let ranks = rank_checked_node_ids(&keys);
        for (left, a) in keys.iter().enumerate() {
            for (right, b) in keys.iter().enumerate() {
                assert_eq!(ranks[left].cmp(&ranks[right]), a.cmp(b));
            }
        }
    }

    /// Trace: FR-038-AC-117, FR-038-AC-380.
    #[test]
    #[ignore = "IR-1191 bounded performance lane"]
    fn sixteen_thousand_nested_ids_rank_within_ten_seconds() {
        use std::sync::Arc;
        use std::time::{Duration, Instant};

        let mut keys = vec![Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![CheckedNodePathSegment::Name("leaf".into())],
            ordinal: 0,
        })];
        for _ in 0..16_000 {
            let child = Arc::clone(keys.last().expect("first key exists"));
            keys.push(Arc::new(CheckedNodeId {
                package: "p".into(),
                path: vec![
                    CheckedNodePathSegment::Name("option".into()),
                    CheckedNodePathSegment::Node(child),
                ],
                ordinal: 0,
            }));
        }
        let start = Instant::now();
        let ranks = rank_checked_node_ids(&keys);
        assert!(
            ranks.into_iter().eq(0..keys.len()),
            "every child ranks before its parent"
        );
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "ranking a 16,000-level shared chain exceeded ten seconds"
        );
    }

    /// Trace: FR-038-AC-364.
    #[test]
    fn structural_identity_rejects_empty_or_out_of_range_components() {
        let mut id = CheckedNodeId {
            package: "example".into(),
            path: vec![CheckedNodePathSegment::Name("A".into())],
            ordinal: 0,
        };
        assert!(id.is_well_formed());
        id.ordinal = 9_007_199_254_740_992;
        assert!(!id.is_well_formed());
        id.ordinal = 0;
        id.path.clear();
        assert!(!id.is_well_formed());
        id.path.push(CheckedNodePathSegment::Name("A".into()));
        id.package = "".into();
        assert!(!id.is_well_formed());
    }

    /// Trace: FR-038-AC-117, FR-038-AC-380.
    #[test]
    fn ten_thousand_rebuilt_node_paths_encode_compare_and_drop_iteratively() {
        const NESTING: usize = 10_000;
        let mut id = std::sync::Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![CheckedNodePathSegment::Name("leaf".into())],
            ordinal: 0,
        });
        for _ in 1..NESTING {
            id = std::sync::Arc::new(CheckedNodeId {
                package: "p".into(),
                path: vec![CheckedNodePathSegment::Node(id)],
                ordinal: 0,
            });
        }
        let id = std::sync::Arc::try_unwrap(id).expect("unique root");
        assert!(id.is_well_formed());
        let copy = id.clone();
        assert_eq!(id.cmp(&copy), std::cmp::Ordering::Equal);
        let canonical = quire_canonical::to_vec(&id, quire_canonical::Limits::new(1 << 20))
            .expect("encode rebuilt structural id");
        assert!(canonical.len() > 100_000);
        // The ordinary drop path also has to consume no native recursion.
        drop(id);
        drop(copy);
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

/// A source document named by the producer, with no byte authentication.
/// The closed shape is `{authority, identity}`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedSourceRef {
    /// Source authority.
    pub authority: Box<str>,
    /// Source identity.
    pub identity: Box<str>,
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

impl Encode for CheckedSourceMapEntry {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        writer.name("node_id")?;
        self.node_id.encode_into(writer)?;
        writer.name("role")?;
        writer.serialize(&self.role)?;
        writer.name("ordinal")?;
        writer.integer(i128::from(self.ordinal))?;
        writer.name("regions")?;
        writer.serialize(&self.regions)?;
        writer.end_object()
    }
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
