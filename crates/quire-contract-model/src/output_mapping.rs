//! Target-neutral output-mapping admission governed by FR-032 and TC-043.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Serialize, Serializer};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    BoundClause, BoundPackage, CanonicalDigest, ClauseKind, ClauseRef, ExecutionPoint, PackageId,
    SchemaVersion, SourceSpan,
};

/// Identity version for the deterministic mapping-request material.
pub const OUTPUT_MAPPING_REQUEST_IDENTITY_VERSION: &str =
    "quire.output.mapping-request-identity/v1-draft.1";
/// Initial FS06 mapping revision shared by all three target profiles.
pub const OUTPUT_MAPPING_REVISION: &str = "1-draft.1";
/// Identity version for per-obligation mapping records.
pub const OUTPUT_MAPPING_RECORD_IDENTITY_VERSION: &str =
    "quire.output.mapping-record-identity/v1-draft.1";

macro_rules! raw_digest_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Construct an already-computed SHA-256 raw digest.
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            /// Hash the exact supplied bytes with raw SHA-256.
            pub fn digest(bytes: &[u8]) -> Self {
                Self(Sha256::digest(bytes).into())
            }

            /// Borrow the 32-byte digest value.
            pub const fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }

            /// Parse exactly 64 lowercase hexadecimal characters.
            pub fn parse(value: &str) -> Result<Self, MappingRequestError> {
                if value.len() != 64
                    || value
                        .bytes()
                        .any(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
                {
                    return Err(MappingRequestError::new(
                        MappingRequestErrorCode::InvalidDigest,
                        "digest",
                        "raw digest must be exactly 64 lowercase hexadecimal characters",
                    ));
                }
                let mut bytes = [0_u8; 32];
                for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
                    bytes[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
                }
                Ok(Self(bytes))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write_digest(self.0, formatter)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_str(self)
            }
        }
    };
}

raw_digest_type!(
    MappingRuleDigest,
    "Raw SHA-256 digest of the exact selected mapping-rule bytes."
);
raw_digest_type!(
    SourceBytesDigest,
    "Raw SHA-256 digest of exact native, model, or semantic selection bytes."
);

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

fn write_digest(bytes: [u8; 32], formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        formatter.write_str(
            std::str::from_utf8(&[HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 0x0f)]])
                .map_err(|_| fmt::Error)?,
        )?;
    }
    Ok(())
}

macro_rules! mapping_error_codes {
    ($( $variant:ident => $wire:literal ),+ $(,)?) => {
        /// Stable machine-readable refusal codes for output-mapping admission.
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        pub enum MappingRequestErrorCode {
            $( #[serde(rename = $wire)] $variant, )+
        }

        impl MappingRequestErrorCode {
            /// Complete closed code catalog.
            pub const ALL: &'static [Self] = &[$( Self::$variant, )+];

            /// Stable wire spelling.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $wire, )+
                }
            }

            /// Resolve one exact stable spelling.
            pub fn from_code(value: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|code| code.as_str() == value)
            }
        }
    };
}

mapping_error_codes! {
    InvalidDigest => "invalid_digest",
    InvalidSourceSelection => "invalid_source_selection",
    UnknownTargetFamily => "unknown_target_family",
    MissingTargetStandard => "missing_target_standard",
    TargetProfileMismatch => "target_profile_mismatch",
    StaleMappingRevision => "stale_mapping_revision",
    UnsupportedCapability => "unsupported_capability",
    DuplicateCapability => "duplicate_capability",
    ZeroLimit => "zero_limit",
    EmptyObligationSelection => "empty_obligation_selection",
    DuplicateObligation => "duplicate_obligation",
    InformationalObligation => "informational_obligation",
    UnknownObligation => "unknown_obligation",
    ForeignObligation => "foreign_obligation",
    StaleObligation => "stale_obligation",
    ObligationOrderMismatch => "obligation_order_mismatch",
    RequestLimitExceeded => "request_limit_exceeded",
    ObligationLimitExceeded => "obligation_limit_exceeded",
    ExpressionNodeLimitExceeded => "expression_node_limit_exceeded",
    NestingDepthLimitExceeded => "nesting_depth_limit_exceeded",
    MappingWorkLimitExceeded => "mapping_work_limit_exceeded",
    RecordLimitExceeded => "record_limit_exceeded",
    EmittedBytesLimitExceeded => "emitted_bytes_limit_exceeded",
    ArithmeticOverflow => "arithmetic_overflow",
    AllocationFailed => "allocation_failed",
    Cancelled => "cancelled",
    InvalidQualifiedReference => "invalid_qualified_reference",
    InvalidOutputRegion => "invalid_output_region",
    DuplicateDependency => "duplicate_dependency",
    InvalidDisposition => "invalid_disposition",
    ZeroMappingWork => "zero_mapping_work",
    CandidateObligationMismatch => "candidate_obligation_mismatch",
    CandidateSourceStateMismatch => "candidate_source_state_mismatch",
}

/// Typed refusal returned before any mapper dispatch or target bytes exist.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MappingRequestError {
    code: MappingRequestErrorCode,
    path: Box<str>,
    message: Box<str>,
}

impl MappingRequestError {
    fn new(
        code: MappingRequestErrorCode,
        path: impl Into<Box<str>>,
        message: impl Into<Box<str>>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }

    /// Stable refusal code.
    pub const fn code(&self) -> MappingRequestErrorCode {
        self.code
    }

    /// Exact affected field path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Human-readable detail; never a source of machine semantics.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for MappingRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code.as_str(),
            self.path,
            self.message
        )
    }
}

impl Error for MappingRequestError {}

/// Closed FS06 output target family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum OutputTargetFamily {
    /// OMG OCL 2.4 output.
    #[serde(rename = "ocl")]
    Ocl,
    /// Inseparable OMG SysML 2.0 and KerML 1.0 output.
    #[serde(rename = "sysml-kerml")]
    SysmlKerml,
    /// NASA FRET Classic FRETish 3.1 output.
    #[serde(rename = "fretish")]
    Fretish,
}

impl OutputTargetFamily {
    /// Parse an exact target-family spelling.
    pub fn parse(value: &str) -> Result<Self, MappingRequestError> {
        match value {
            "ocl" => Ok(Self::Ocl),
            "sysml-kerml" => Ok(Self::SysmlKerml),
            "fretish" => Ok(Self::Fretish),
            _ => Err(MappingRequestError::new(
                MappingRequestErrorCode::UnknownTargetFamily,
                "profile.target_family",
                "target family is not an accepted FS06 family",
            )),
        }
    }

    /// Exact target-family spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ocl => "ocl",
            Self::SysmlKerml => "sysml-kerml",
            Self::Fretish => "fretish",
        }
    }
}

/// Closed requested mapping capability catalog shared by the initial profiles.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputCapability {
    /// Total Boolean literals and connectives.
    Boolean,
    /// Bounded checked integer expressions.
    BoundedInteger,
    /// The bounded ConfigVersion scalar.
    ConfigVersion,
    /// Exact field reads.
    FieldRead,
    /// Exact operation pre/post contracts.
    OperationContract,
    /// Ordered duplicate-preserving Sequence queries.
    SequenceQuery,
    /// Exact SysML/KerML attribute correspondence.
    AttributeReference,
    /// SysML requirement/constraint structure.
    RequirementConstraint,
    /// FRETish state-predicate form.
    StatePredicate,
    /// FRETish same-sample response.
    ImmediateResponse,
    /// FRETish fixed-sample bounded eventual response.
    BoundedEventualResponse,
    /// FRETish exact next-timepoint response.
    NextTimepointResponse,
}

impl OutputCapability {
    /// Stable capability spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::BoundedInteger => "bounded-integer",
            Self::ConfigVersion => "config-version",
            Self::FieldRead => "field-read",
            Self::OperationContract => "operation-contract",
            Self::SequenceQuery => "sequence-query",
            Self::AttributeReference => "attribute-reference",
            Self::RequirementConstraint => "requirement-constraint",
            Self::StatePredicate => "state-predicate",
            Self::ImmediateResponse => "immediate-response",
            Self::BoundedEventualResponse => "bounded-eventual-response",
            Self::NextTimepointResponse => "next-timepoint-response",
        }
    }

    /// Parse one exact closed capability spelling.
    pub fn parse(value: &str) -> Result<Self, MappingRequestError> {
        [
            Self::Boolean,
            Self::BoundedInteger,
            Self::ConfigVersion,
            Self::FieldRead,
            Self::OperationContract,
            Self::SequenceQuery,
            Self::AttributeReference,
            Self::RequirementConstraint,
            Self::StatePredicate,
            Self::ImmediateResponse,
            Self::BoundedEventualResponse,
            Self::NextTimepointResponse,
        ]
        .into_iter()
        .find(|capability| capability.as_str() == value)
        .ok_or_else(|| {
            MappingRequestError::new(
                MappingRequestErrorCode::UnsupportedCapability,
                "profile.required_capabilities",
                "capability is not in the closed FS06 catalog",
            )
        })
    }

    const fn accepted_by(self, family: OutputTargetFamily) -> bool {
        match family {
            OutputTargetFamily::Ocl => matches!(
                self,
                Self::Boolean
                    | Self::BoundedInteger
                    | Self::ConfigVersion
                    | Self::FieldRead
                    | Self::OperationContract
                    | Self::SequenceQuery
            ),
            OutputTargetFamily::SysmlKerml => matches!(
                self,
                Self::Boolean
                    | Self::BoundedInteger
                    | Self::ConfigVersion
                    | Self::AttributeReference
                    | Self::RequirementConstraint
            ),
            OutputTargetFamily::Fretish => matches!(
                self,
                Self::Boolean
                    | Self::BoundedInteger
                    | Self::ConfigVersion
                    | Self::StatePredicate
                    | Self::ImmediateResponse
                    | Self::BoundedEventualResponse
                    | Self::NextTimepointResponse
            ),
        }
    }
}

/// Exact, observer-independent FS06 target profile selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OutputMappingProfile {
    target_family: OutputTargetFamily,
    target_standard_refs: Vec<&'static str>,
    mapping_profile_id: &'static str,
    mapping_revision: &'static str,
    mapping_digest: MappingRuleDigest,
    required_capabilities: Vec<OutputCapability>,
}

impl OutputMappingProfile {
    /// Validate and construct one exact FS06 family/profile selection.
    pub fn new(
        target_family: &str,
        target_standard_refs: Vec<&str>,
        mapping_profile_id: &str,
        mapping_revision: &str,
        mapping_digest: MappingRuleDigest,
        required_capabilities: Vec<OutputCapability>,
    ) -> Result<Self, MappingRequestError> {
        let family = OutputTargetFamily::parse(target_family)?;
        let (accepted_refs, accepted_id): (&[&str], &str) = match family {
            OutputTargetFamily::Ocl => (&["formal/14-02-03"], "quire.output.ocl24/v1"),
            OutputTargetFamily::SysmlKerml => (
                &["formal/26-03-02", "formal/26-03-01"],
                "quire.output.sysml2-kerml1/v1",
            ),
            OutputTargetFamily::Fretish => (&["v3.1.0"], "quire.output.fretish31/v1"),
        };
        if target_standard_refs.is_empty() {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::MissingTargetStandard,
                "profile.target_standard_refs",
                "target standard reference list must not be empty",
            ));
        }
        if target_standard_refs != accepted_refs || mapping_profile_id != accepted_id {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::TargetProfileMismatch,
                "profile",
                "target family, ordered standards, and profile identity do not match",
            ));
        }
        if mapping_revision != OUTPUT_MAPPING_REVISION {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::StaleMappingRevision,
                "profile.mapping_revision",
                "mapping revision is not the accepted FS06 revision",
            ));
        }
        let mut seen = BTreeSet::new();
        for capability in &required_capabilities {
            if !seen.insert(*capability) {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::DuplicateCapability,
                    "profile.required_capabilities",
                    "requested capability occurs more than once",
                ));
            }
            if !capability.accepted_by(family) {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::UnsupportedCapability,
                    "profile.required_capabilities",
                    "requested capability belongs to a different target profile",
                ));
            }
        }
        let required_capabilities = seen.into_iter().collect();
        let target_standard_refs = accepted_refs.to_vec();
        let mapping_profile_id = match family {
            OutputTargetFamily::Ocl => "quire.output.ocl24/v1",
            OutputTargetFamily::SysmlKerml => "quire.output.sysml2-kerml1/v1",
            OutputTargetFamily::Fretish => "quire.output.fretish31/v1",
        };
        Ok(Self {
            target_family: family,
            target_standard_refs,
            mapping_profile_id,
            mapping_revision: OUTPUT_MAPPING_REVISION,
            mapping_digest,
            required_capabilities,
        })
    }

    /// Selected target family.
    pub const fn target_family(&self) -> OutputTargetFamily {
        self.target_family
    }

    /// Ordered normative target revisions.
    pub fn target_standard_refs(&self) -> &[&'static str] {
        &self.target_standard_refs
    }

    /// Exact Quire mapping profile identity.
    pub const fn mapping_profile_id(&self) -> &'static str {
        self.mapping_profile_id
    }

    /// Exact mapping revision.
    pub const fn mapping_revision(&self) -> &'static str {
        self.mapping_revision
    }

    /// Raw mapping-rule digest.
    pub const fn mapping_digest(&self) -> MappingRuleDigest {
        self.mapping_digest
    }

    /// Canonically ordered closed capability set.
    pub fn required_capabilities(&self) -> &[OutputCapability] {
        &self.required_capabilities
    }
}

fn valid_selection_member(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.bytes().all(|byte| byte.is_ascii_graphic())
}

macro_rules! source_selection_type {
    ($name:ident, $doc:literal, $path:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
        pub struct $name {
            identity: Box<str>,
            revision: Box<str>,
            digest: SourceBytesDigest,
        }

        impl $name {
            /// Construct an exact independently typed source selection.
            pub fn new(
                identity: impl Into<String>,
                revision: impl Into<String>,
                digest: SourceBytesDigest,
            ) -> Result<Self, MappingRequestError> {
                let identity = identity.into();
                let revision = revision.into();
                if !valid_selection_member(&identity) || !valid_selection_member(&revision) {
                    return Err(MappingRequestError::new(
                        MappingRequestErrorCode::InvalidSourceSelection,
                        $path,
                        "selection identity and revision must be nonempty bounded visible ASCII",
                    ));
                }
                Ok(Self {
                    identity: identity.into_boxed_str(),
                    revision: revision.into_boxed_str(),
                    digest,
                })
            }

            /// Exact selected contract identity.
            pub fn identity(&self) -> &str {
                &self.identity
            }

            /// Exact immutable selected revision.
            pub fn revision(&self) -> &str {
                &self.revision
            }

            /// Raw digest of the selected bytes.
            pub const fn digest(&self) -> SourceBytesDigest {
                self.digest
            }
        }
    };
}

source_selection_type!(
    NativeSourceSelection,
    "Exact native-source contract selection.",
    "selections.native"
);
source_selection_type!(
    ModelSourceSelection,
    "Exact authoritative model contract selection.",
    "selections.model"
);
source_selection_type!(
    SemanticSourceSelection,
    "Exact semantic-profile contract selection.",
    "selections.semantic"
);

/// Explicit aggregate resource ceilings for one mapping request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MappingLimits {
    maximum_request_bytes: u64,
    maximum_obligations: u64,
    maximum_expression_nodes: u64,
    maximum_nesting_depth: u64,
    maximum_mapping_work: u64,
    maximum_records: u64,
    maximum_emitted_bytes: u64,
}

impl MappingLimits {
    /// Construct a complete limit set; no zero/default capacity is admitted.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        maximum_request_bytes: u64,
        maximum_obligations: u64,
        maximum_expression_nodes: u64,
        maximum_nesting_depth: u64,
        maximum_mapping_work: u64,
        maximum_records: u64,
        maximum_emitted_bytes: u64,
    ) -> Result<Self, MappingRequestError> {
        if [
            maximum_request_bytes,
            maximum_obligations,
            maximum_expression_nodes,
            maximum_nesting_depth,
            maximum_mapping_work,
            maximum_records,
            maximum_emitted_bytes,
        ]
        .contains(&0)
        {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::ZeroLimit,
                "limits",
                "every output-mapping limit must be positive",
            ));
        }
        Ok(Self {
            maximum_request_bytes,
            maximum_obligations,
            maximum_expression_nodes,
            maximum_nesting_depth,
            maximum_mapping_work,
            maximum_records,
            maximum_emitted_bytes,
        })
    }

    /// Maximum canonical semantic request bytes.
    pub const fn maximum_request_bytes(&self) -> u64 {
        self.maximum_request_bytes
    }
    /// Maximum selected obligations.
    pub const fn maximum_obligations(&self) -> u64 {
        self.maximum_obligations
    }
    /// Maximum aggregate checked expression nodes.
    pub const fn maximum_expression_nodes(&self) -> u64 {
        self.maximum_expression_nodes
    }
    /// Maximum nesting depth of any selected expression.
    pub const fn maximum_nesting_depth(&self) -> u64 {
        self.maximum_nesting_depth
    }
    /// Maximum target mapper work units.
    pub const fn maximum_mapping_work(&self) -> u64 {
        self.maximum_mapping_work
    }
    /// Maximum mapping records.
    pub const fn maximum_records(&self) -> u64 {
        self.maximum_records
    }
    /// Maximum emitted target bytes.
    pub const fn maximum_emitted_bytes(&self) -> u64 {
        self.maximum_emitted_bytes
    }
}

/// Closed dependency domains retained by a mapping record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingDependencyKind {
    /// Native or target semantic selection.
    Semantic,
    /// Authoritative model selection.
    Model,
    /// Type correspondence.
    Type,
    /// Unit correspondence.
    Unit,
    /// Execution-anchor correspondence.
    Anchor,
    /// Capture identity or instant.
    Capture,
    /// Clock correspondence.
    Clock,
    /// History correspondence.
    History,
    /// Observation result or authority fact.
    Observation,
    /// Protocol or other selected result.
    Result,
}

/// Exact owner-qualified dependency actually used for one mapping decision.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct MappingDependencyRef {
    kind: MappingDependencyKind,
    owner: Box<str>,
    identity: Box<str>,
    revision: Box<str>,
    digest: SourceBytesDigest,
}

impl MappingDependencyRef {
    /// Construct an exact typed dependency reference.
    pub fn new(
        kind: MappingDependencyKind,
        owner: impl Into<String>,
        identity: impl Into<String>,
        revision: impl Into<String>,
        digest: SourceBytesDigest,
    ) -> Result<Self, MappingRequestError> {
        let owner = owner.into();
        let identity = identity.into();
        let revision = revision.into();
        validate_qualified_members(&owner, &identity, &revision, "dependencies")?;
        Ok(Self {
            kind,
            owner: owner.into_boxed_str(),
            identity: identity.into_boxed_str(),
            revision: revision.into_boxed_str(),
            digest,
        })
    }

    /// Dependency domain.
    pub const fn kind(&self) -> MappingDependencyKind {
        self.kind
    }

    /// Exact owner identity.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Exact owner-local contract or fact identity.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Exact immutable owner revision.
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Raw digest of the selected owner bytes.
    pub const fn digest(&self) -> SourceBytesDigest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct QualifiedMappingCode {
    owner: Box<str>,
    contract: Box<str>,
    revision: Box<str>,
    digest: SourceBytesDigest,
    code: Box<str>,
}

impl QualifiedMappingCode {
    fn new(
        owner: impl Into<String>,
        contract: impl Into<String>,
        revision: impl Into<String>,
        digest: SourceBytesDigest,
        code: impl Into<String>,
        path: &'static str,
    ) -> Result<Self, MappingRequestError> {
        let owner = owner.into();
        let contract = contract.into();
        let revision = revision.into();
        let code = code.into();
        validate_qualified_members(&owner, &contract, &revision, path)?;
        if !valid_selection_member(&code) {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::InvalidQualifiedReference,
                path,
                "qualified condition or cause code is invalid",
            ));
        }
        Ok(Self {
            owner: owner.into_boxed_str(),
            contract: contract.into_boxed_str(),
            revision: revision.into_boxed_str(),
            digest,
            code: code.into_boxed_str(),
        })
    }
}

macro_rules! qualified_code_type {
    ($name:ident, $doc:literal, $path:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(QualifiedMappingCode);

        impl $name {
            /// Construct an exact owner-qualified machine code.
            pub fn new(
                owner: impl Into<String>,
                contract: impl Into<String>,
                revision: impl Into<String>,
                digest: SourceBytesDigest,
                code: impl Into<String>,
            ) -> Result<Self, MappingRequestError> {
                QualifiedMappingCode::new(owner, contract, revision, digest, code, $path).map(Self)
            }

            /// Exact owner identity.
            pub fn owner(&self) -> &str {
                &self.0.owner
            }

            /// Exact selected contract identity.
            pub fn contract(&self) -> &str {
                &self.0.contract
            }

            /// Exact immutable contract revision.
            pub fn revision(&self) -> &str {
                &self.0.revision
            }

            /// Raw digest of the selected contract bytes.
            pub const fn digest(&self) -> SourceBytesDigest {
                self.0.digest
            }

            /// Exact machine-readable code.
            pub fn code(&self) -> &str {
                &self.0.code
            }
        }
    };
}

qualified_code_type!(
    MappingCondition,
    "Exact retained condition under which a conditional mapping holds.",
    "candidate.conditions"
);
qualified_code_type!(
    MappingCause,
    "Exact retained cause for unrepresented or refused mapping behavior.",
    "candidate.causes"
);

/// Closed observation-adequacy vocabulary from FR-245.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ObservationAdequacyState {
    /// Required observations are present.
    #[serde(rename = "adequate")]
    Adequate,
    /// Required observations are missing.
    #[serde(rename = "inadequate")]
    Inadequate,
    /// Adequacy is unknown.
    #[serde(rename = "unknown")]
    Unknown,
    /// Adequacy was not assessed.
    #[serde(rename = "not-assessed")]
    NotAssessed,
}

/// Exact owner-qualified observation-adequacy fact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ObservationAdequacyRef {
    owner: Box<str>,
    contract: Box<str>,
    revision: Box<str>,
    digest: SourceBytesDigest,
    state: ObservationAdequacyState,
}

impl ObservationAdequacyRef {
    /// Construct a qualified observation-adequacy fact.
    pub fn new(
        owner: impl Into<String>,
        contract: impl Into<String>,
        revision: impl Into<String>,
        digest: SourceBytesDigest,
        state: ObservationAdequacyState,
    ) -> Result<Self, MappingRequestError> {
        let owner = owner.into();
        let contract = contract.into();
        let revision = revision.into();
        validate_qualified_members(&owner, &contract, &revision, "observation_adequacy")?;
        Ok(Self {
            owner: owner.into_boxed_str(),
            contract: contract.into_boxed_str(),
            revision: revision.into_boxed_str(),
            digest,
            state,
        })
    }

    /// Exact adequacy state.
    pub const fn state(&self) -> ObservationAdequacyState {
        self.state
    }

    /// Exact owner identity.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Exact selected contract identity.
    pub fn contract(&self) -> &str {
        &self.contract
    }

    /// Exact immutable contract revision.
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Raw digest of the selected contract bytes.
    pub const fn digest(&self) -> SourceBytesDigest {
        self.digest
    }
}

/// Closed protocol-adequacy vocabulary from FR-246.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ProtocolAdequacyState {
    /// Exact declared obligation/case set was demonstrated.
    #[serde(rename = "demonstrated")]
    Demonstrated,
    /// Exact declared obligation/case set was not demonstrated.
    #[serde(rename = "not-demonstrated")]
    NotDemonstrated,
    /// Declared obligation/case set does not apply.
    #[serde(rename = "inapplicable")]
    Inapplicable,
    /// Adequacy is unknown.
    #[serde(rename = "unknown")]
    Unknown,
}

/// Exact owner-qualified protocol-adequacy fact, disjoint by type from observation adequacy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProtocolAdequacyRef {
    owner: Box<str>,
    contract: Box<str>,
    revision: Box<str>,
    digest: SourceBytesDigest,
    state: ProtocolAdequacyState,
}

impl ProtocolAdequacyRef {
    /// Construct a qualified protocol-adequacy fact.
    pub fn new(
        owner: impl Into<String>,
        contract: impl Into<String>,
        revision: impl Into<String>,
        digest: SourceBytesDigest,
        state: ProtocolAdequacyState,
    ) -> Result<Self, MappingRequestError> {
        let owner = owner.into();
        let contract = contract.into();
        let revision = revision.into();
        validate_qualified_members(&owner, &contract, &revision, "protocol_adequacy")?;
        Ok(Self {
            owner: owner.into_boxed_str(),
            contract: contract.into_boxed_str(),
            revision: revision.into_boxed_str(),
            digest,
            state,
        })
    }

    /// Exact adequacy state.
    pub const fn state(&self) -> ProtocolAdequacyState {
        self.state
    }

    /// Exact owner identity.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Exact selected contract identity.
    pub fn contract(&self) -> &str {
        &self.contract
    }

    /// Exact immutable contract revision.
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Raw digest of the selected contract bytes.
    pub const fn digest(&self) -> SourceBytesDigest {
        self.digest
    }
}

/// Closed per-obligation mapping disposition from FR-269.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingDisposition {
    /// Every required fact is represented without retained semantic loss.
    Preserved,
    /// Representation is valid only under retained conditions.
    Conditional,
    /// Source fact is retained but has no target representation.
    Unrepresented,
    /// Mapping was explicitly refused with no substitute output.
    Refused,
}

/// One nonempty half-open target byte region.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct OutputByteRegion {
    start: u64,
    end: u64,
}

impl OutputByteRegion {
    /// Construct a nonempty half-open byte interval.
    pub fn new(start: u64, end: u64) -> Result<Self, MappingRequestError> {
        if start >= end {
            Err(MappingRequestError::new(
                MappingRequestErrorCode::InvalidOutputRegion,
                "candidate.output_regions",
                "output region must be a nonempty increasing half-open interval",
            ))
        } else {
            Ok(Self { start, end })
        }
    }

    /// Inclusive start byte offset.
    pub const fn start(self) -> u64 {
        self.start
    }

    /// Exclusive end byte offset.
    pub const fn end(self) -> u64 {
        self.end
    }

    fn shifted(self, offset: u64) -> Result<Self, MappingRequestError> {
        let start = self.start.checked_add(offset).ok_or_else(|| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "record.output_regions.start",
                "absolute output-region start overflowed",
            )
        })?;
        let end = self.end.checked_add(offset).ok_or_else(|| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "record.output_regions.end",
                "absolute output-region end overflowed",
            )
        })?;
        Self::new(start, end)
    }
}

/// Source assessment state retained independently from mapping disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFactState {
    /// The exact source fact is available for mapping.
    Ready,
    /// The source assessment remains pending.
    Pending,
    /// The source assessment is incomplete.
    Incomplete,
    /// The source assessment was refused.
    Refused,
}

/// One exact requested source obligation and its retained source state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RequestedMappingObligation {
    identity: ClauseRef,
    source_state: SourceFactState,
}

impl RequestedMappingObligation {
    /// Construct a typed request member; package resolution occurs on admission.
    pub fn new(identity: ClauseRef, source_state: SourceFactState) -> Self {
        Self {
            identity,
            source_state,
        }
    }

    /// Exact requested clause identity.
    pub fn identity(&self) -> &ClauseRef {
        &self.identity
    }

    /// Exact retained source assessment state.
    pub const fn source_state(&self) -> SourceFactState {
        self.source_state
    }
}

/// Request-time cancellation state, checked before any mapper dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingCancellation {
    /// Continue admission.
    Active,
    /// Refuse admission without exposing target output.
    Cancelled,
}

/// Exact source-package reference retained by an admitted request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MappingSourcePackageRef {
    package: PackageId,
    schema_version: SchemaVersion,
    digest: CanonicalDigest,
}

impl MappingSourcePackageRef {
    /// Source package namespace.
    pub fn package(&self) -> &PackageId {
        &self.package
    }

    /// Strict package schema version.
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// Recomputed bound-package identity.
    pub const fn digest(&self) -> CanonicalDigest {
        self.digest
    }
}

/// Immutable obligation view passed to a target mapper after admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedMappingObligation {
    clause: BoundClause,
    source_state: SourceFactState,
}

impl AdmittedMappingObligation {
    /// Exact source clause identity.
    pub fn identity(&self) -> &ClauseRef {
        self.clause.identity()
    }

    /// Strict checked bound clause.
    pub fn clause(&self) -> &BoundClause {
        &self.clause
    }

    /// Exact retained source state.
    pub const fn source_state(&self) -> SourceFactState {
        self.source_state
    }
}

/// Remaining aggregate mapping-work budget presented to one mapper invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MappingWorkBudget {
    remaining: u64,
}

impl MappingWorkBudget {
    /// Work units still available before this invocation.
    pub const fn remaining(self) -> u64 {
        self.remaining
    }
}

/// Target-neutral candidate returned by exactly one selected mapper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingCandidate {
    obligation: ClauseRef,
    source_state: SourceFactState,
    fragment: Vec<u8>,
    local_regions: Vec<OutputByteRegion>,
    dependencies: Vec<MappingDependencyRef>,
    disposition: MappingDisposition,
    conditions: Vec<MappingCondition>,
    causes: Vec<MappingCause>,
    observation_adequacy: Option<ObservationAdequacyRef>,
    protocol_adequacy: Option<ProtocolAdequacyRef>,
    work: u64,
}

impl MappingCandidate {
    /// Validate one local mapper result before common accounting.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        obligation: ClauseRef,
        source_state: SourceFactState,
        fragment: Vec<u8>,
        local_regions: Vec<OutputByteRegion>,
        dependencies: Vec<MappingDependencyRef>,
        disposition: MappingDisposition,
        conditions: Vec<MappingCondition>,
        causes: Vec<MappingCause>,
        observation_adequacy: Option<ObservationAdequacyRef>,
        protocol_adequacy: Option<ProtocolAdequacyRef>,
        work: u64,
    ) -> Result<Self, MappingRequestError> {
        if work == 0 {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::ZeroMappingWork,
                "candidate.work",
                "every mapper invocation must account for positive work",
            ));
        }
        let fragment_text = std::str::from_utf8(&fragment).map_err(|_| {
            MappingRequestError::new(
                MappingRequestErrorCode::InvalidOutputRegion,
                "candidate.fragment",
                "initial FS06 target fragments must be UTF-8",
            )
        })?;
        let fragment_len = u64::try_from(fragment.len()).map_err(|_| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "candidate.fragment",
                "fragment byte count exceeds the supported integer range",
            )
        })?;
        let mut previous_end = 0_u64;
        for region in &local_regions {
            if region.end > fragment_len || region.start < previous_end {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::InvalidOutputRegion,
                    "candidate.output_regions",
                    "local regions must be ordered, nonoverlapping, and within the fragment",
                ));
            }
            let start = usize::try_from(region.start).map_err(|_| {
                MappingRequestError::new(
                    MappingRequestErrorCode::ArithmeticOverflow,
                    "candidate.output_regions.start",
                    "region start exceeds the platform index range",
                )
            })?;
            let end = usize::try_from(region.end).map_err(|_| {
                MappingRequestError::new(
                    MappingRequestErrorCode::ArithmeticOverflow,
                    "candidate.output_regions.end",
                    "region end exceeds the platform index range",
                )
            })?;
            if !fragment_text.is_char_boundary(start) || !fragment_text.is_char_boundary(end) {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::InvalidOutputRegion,
                    "candidate.output_regions",
                    "output regions must align to UTF-8 code point boundaries",
                ));
            }
            previous_end = region.end;
        }
        let mut unique_dependencies = BTreeSet::new();
        if dependencies
            .iter()
            .any(|dependency| !unique_dependencies.insert(dependency.clone()))
        {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::DuplicateDependency,
                "candidate.dependencies",
                "candidate dependencies must be exact and unique",
            ));
        }
        let represented = !fragment.is_empty() && !local_regions.is_empty();
        let source_ready = source_state == SourceFactState::Ready;
        let valid_disposition = match disposition {
            MappingDisposition::Preserved => {
                source_ready && represented && conditions.is_empty() && causes.is_empty()
            }
            MappingDisposition::Conditional => {
                source_ready && represented && !conditions.is_empty()
            }
            MappingDisposition::Unrepresented | MappingDisposition::Refused => {
                fragment.is_empty()
                    && local_regions.is_empty()
                    && conditions.is_empty()
                    && !causes.is_empty()
            }
        };
        if !valid_disposition {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::InvalidDisposition,
                "candidate.disposition",
                "candidate disposition, source state, output, conditions, and causes disagree",
            ));
        }
        Ok(Self {
            obligation,
            source_state,
            fragment,
            local_regions,
            dependencies,
            disposition,
            conditions,
            causes,
            observation_adequacy,
            protocol_adequacy,
            work,
        })
    }

    /// Exact source obligation claimed by this candidate.
    pub fn obligation(&self) -> &ClauseRef {
        &self.obligation
    }

    /// Retained source fact state.
    pub const fn source_state(&self) -> SourceFactState {
        self.source_state
    }

    /// Deterministic UTF-8 target fragment bytes.
    pub fn fragment(&self) -> &[u8] {
        &self.fragment
    }

    /// Ordered local half-open byte regions.
    pub fn local_regions(&self) -> &[OutputByteRegion] {
        &self.local_regions
    }

    /// Exact dependencies actually used.
    pub fn dependencies(&self) -> &[MappingDependencyRef] {
        &self.dependencies
    }

    /// Explicit mapping disposition.
    pub const fn disposition(&self) -> MappingDisposition {
        self.disposition
    }

    /// Ordered retained conditions.
    pub fn conditions(&self) -> &[MappingCondition] {
        &self.conditions
    }

    /// Ordered retained causes.
    pub fn causes(&self) -> &[MappingCause] {
        &self.causes
    }

    /// Separately typed observation-adequacy fact, when supplied.
    pub fn observation_adequacy(&self) -> Option<&ObservationAdequacyRef> {
        self.observation_adequacy.as_ref()
    }

    /// Separately typed protocol-adequacy fact, when supplied.
    pub fn protocol_adequacy(&self) -> Option<&ProtocolAdequacyRef> {
        self.protocol_adequacy.as_ref()
    }

    /// Charged mapper work units.
    pub const fn work(&self) -> u64 {
        self.work
    }
}

/// Target-specific correspondence seam consumed by the common coordinator.
pub trait OutputMapper {
    /// Exact profile implemented by this mapper instance.
    fn profile(&self) -> &OutputMappingProfile;

    /// Map one admitted obligation exactly once within the presented budget.
    fn map_obligation(
        &mut self,
        obligation: &AdmittedMappingObligation,
        budget: MappingWorkBudget,
    ) -> Result<MappingCandidate, MappingRequestError>;
}

raw_digest_type!(
    MappingRecordId,
    "Derived SHA-256-over-JCS identity of one complete mapping record."
);

/// Exact source identity and semantic content bound into a mapping record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MappingRecordSource {
    identity: ClauseRef,
    kind: ClauseKind,
    anchor: ExecutionPoint,
    source: SourceSpan,
    declaration_digest: CanonicalDigest,
    expression_digest: CanonicalDigest,
}

impl MappingRecordSource {
    fn from_clause(clause: &BoundClause) -> Self {
        Self {
            identity: clause.identity().clone(),
            kind: clause.kind(),
            anchor: clause.anchor().clone(),
            source: clause.source().clone(),
            declaration_digest: clause.declaration_digest(),
            expression_digest: clause.expression_digest(),
        }
    }

    /// Exact source clause identity.
    pub fn identity(&self) -> &ClauseRef {
        &self.identity
    }

    /// Exact executable clause kind.
    pub const fn kind(&self) -> ClauseKind {
        self.kind
    }

    /// Exact execution anchor.
    pub fn anchor(&self) -> &ExecutionPoint {
        &self.anchor
    }

    /// Exact source region.
    pub fn source(&self) -> &SourceSpan {
        &self.source
    }

    /// Canonical declaration-environment digest.
    pub const fn declaration_digest(&self) -> CanonicalDigest {
        self.declaration_digest
    }

    /// Canonical checked-expression digest.
    pub const fn expression_digest(&self) -> CanonicalDigest {
        self.expression_digest
    }
}

/// Immutable, identity-bearing FR-298 mapping record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OutputMappingRecord {
    record_id: MappingRecordId,
    source: MappingRecordSource,
    source_state: SourceFactState,
    dependencies: Vec<MappingDependencyRef>,
    target_profile: OutputMappingProfile,
    disposition: MappingDisposition,
    conditions: Vec<MappingCondition>,
    causes: Vec<MappingCause>,
    output_regions: Vec<OutputByteRegion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observation_adequacy: Option<ObservationAdequacyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_adequacy: Option<ProtocolAdequacyRef>,
}

impl OutputMappingRecord {
    /// Derived record identity.
    pub const fn record_id(&self) -> MappingRecordId {
        self.record_id
    }

    /// Exact source obligation and semantic content.
    pub fn source(&self) -> &MappingRecordSource {
        &self.source
    }

    /// Retained source assessment state.
    pub const fn source_state(&self) -> SourceFactState {
        self.source_state
    }

    /// Exact ordered dependencies actually used.
    pub fn dependencies(&self) -> &[MappingDependencyRef] {
        &self.dependencies
    }

    /// Exact target profile.
    pub fn target_profile(&self) -> &OutputMappingProfile {
        &self.target_profile
    }

    /// Explicit loss disposition.
    pub const fn disposition(&self) -> MappingDisposition {
        self.disposition
    }

    /// Ordered retained conditions.
    pub fn conditions(&self) -> &[MappingCondition] {
        &self.conditions
    }

    /// Ordered retained causes.
    pub fn causes(&self) -> &[MappingCause] {
        &self.causes
    }

    /// Ordered absolute half-open output regions.
    pub fn output_regions(&self) -> &[OutputByteRegion] {
        &self.output_regions
    }

    /// Separately typed observation adequacy, when present.
    pub fn observation_adequacy(&self) -> Option<&ObservationAdequacyRef> {
        self.observation_adequacy.as_ref()
    }

    /// Separately typed protocol adequacy, when present.
    pub fn protocol_adequacy(&self) -> Option<&ProtocolAdequacyRef> {
        self.protocol_adequacy.as_ref()
    }
}

/// Complete all-or-nothing candidate and record population ready for assembly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedMappings {
    request: AdmittedMappingRequest,
    records: Vec<OutputMappingRecord>,
    fragments: Vec<Vec<u8>>,
    mapping_work: u64,
    emitted_bytes: u64,
}

impl CompletedMappings {
    /// Exact admitted request that produced this complete population.
    pub fn request(&self) -> &AdmittedMappingRequest {
        &self.request
    }

    /// One record for every admitted obligation in source order.
    pub fn records(&self) -> &[OutputMappingRecord] {
        &self.records
    }

    /// Exact aggregate work charged by all mapper invocations.
    pub const fn mapping_work(&self) -> u64 {
        self.mapping_work
    }

    /// Exact aggregate fragment bytes awaiting atomic assembly.
    pub const fn emitted_bytes(&self) -> u64 {
        self.emitted_bytes
    }
}

/// Fully validated target-neutral request; construction is atomic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedMappingRequest {
    package: BoundPackage,
    source_package: MappingSourcePackageRef,
    obligations: Vec<AdmittedMappingObligation>,
    native: NativeSourceSelection,
    model: ModelSourceSelection,
    semantic: SemanticSourceSelection,
    profile: OutputMappingProfile,
    limits: MappingLimits,
    request_bytes: u64,
    expression_nodes: u64,
    nesting_depth: u64,
}

impl AdmittedMappingRequest {
    /// Validate every source, profile, ordering, allocation, cancellation, and
    /// aggregate resource invariant before returning a request.
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        package: &BoundPackage,
        requested: Vec<RequestedMappingObligation>,
        native: NativeSourceSelection,
        model: ModelSourceSelection,
        semantic: SemanticSourceSelection,
        profile: OutputMappingProfile,
        limits: MappingLimits,
        cancellation: MappingCancellation,
    ) -> Result<Self, MappingRequestError> {
        if cancellation == MappingCancellation::Cancelled {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::Cancelled,
                "request",
                "mapping request was cancelled before admission",
            ));
        }
        if requested.is_empty() {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::EmptyObligationSelection,
                "request.obligations",
                "at least one executable obligation must be selected",
            ));
        }
        let obligation_count = u64::try_from(requested.len()).map_err(|_| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "request.obligations",
                "obligation count exceeds the supported integer range",
            )
        })?;
        check_limit(
            obligation_count,
            limits.maximum_obligations,
            MappingRequestErrorCode::ObligationLimitExceeded,
            "request.obligations",
        )?;
        check_limit(
            obligation_count,
            limits.maximum_mapping_work,
            MappingRequestErrorCode::MappingWorkLimitExceeded,
            "request.obligations",
        )?;
        check_limit(
            obligation_count,
            limits.maximum_records,
            MappingRequestErrorCode::RecordLimitExceeded,
            "request.obligations",
        )?;

        let mut obligations = Vec::new();
        obligations
            .try_reserve_exact(requested.len())
            .map_err(|_| {
                MappingRequestError::new(
                    MappingRequestErrorCode::AllocationFailed,
                    "request.obligations",
                    "obligation allocation failed",
                )
            })?;
        let mut seen = BTreeSet::new();
        let mut previous_order = None;
        let mut expression_nodes = 0_u64;
        let mut nesting_depth = 0_u64;
        for requested_obligation in &requested {
            if !seen.insert(requested_obligation.identity.clone()) {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::DuplicateObligation,
                    "request.obligations",
                    "an obligation identity occurs more than once",
                ));
            }
            if package
                .informational()
                .binary_search(&requested_obligation.identity)
                .is_ok()
            {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::InformationalObligation,
                    "request.obligations",
                    "informational clauses cannot be mapped as executable obligations",
                ));
            }
            let clause = package
                .clauses()
                .iter()
                .find(|clause| clause.identity() == &requested_obligation.identity)
                .ok_or_else(|| classify_unknown(package, &requested_obligation.identity))?;
            if previous_order.is_some_and(|order| clause.source_order() <= order) {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::ObligationOrderMismatch,
                    "request.obligations",
                    "obligations must retain authored source order",
                ));
            }
            previous_order = Some(clause.source_order());
            let nodes = u64::try_from(clause.expression().nodes().len()).map_err(|_| {
                MappingRequestError::new(
                    MappingRequestErrorCode::ArithmeticOverflow,
                    "request.expression_nodes",
                    "expression node count exceeds the supported integer range",
                )
            })?;
            expression_nodes = expression_nodes.checked_add(nodes).ok_or_else(|| {
                MappingRequestError::new(
                    MappingRequestErrorCode::ArithmeticOverflow,
                    "request.expression_nodes",
                    "aggregate expression node count overflowed",
                )
            })?;
            nesting_depth = nesting_depth.max(u64::from(clause.expression().nesting_depth()));
            obligations.push(AdmittedMappingObligation {
                clause: clause.clone(),
                source_state: requested_obligation.source_state,
            });
        }
        check_limit(
            expression_nodes,
            limits.maximum_expression_nodes,
            MappingRequestErrorCode::ExpressionNodeLimitExceeded,
            "request.expression_nodes",
        )?;
        check_limit(
            nesting_depth,
            limits.maximum_nesting_depth,
            MappingRequestErrorCode::NestingDepthLimitExceeded,
            "request.nesting_depth",
        )?;

        let source_package = MappingSourcePackageRef {
            package: package.package().id().clone(),
            schema_version: package.package().schema_version(),
            digest: package.digest(),
        };
        let request_material = json!({
            "identity_version": OUTPUT_MAPPING_REQUEST_IDENTITY_VERSION,
            "source_package": &source_package,
            "obligations": requested,
            "native_selection": &native,
            "model_selection": &model,
            "semantic_selection": &semantic,
            "target_profile": &profile,
            "resource_shape": {
                "maximum_obligations": limits.maximum_obligations,
                "maximum_expression_nodes": limits.maximum_expression_nodes,
                "maximum_nesting_depth": limits.maximum_nesting_depth,
                "maximum_mapping_work": limits.maximum_mapping_work,
                "maximum_records": limits.maximum_records,
                "maximum_emitted_bytes": limits.maximum_emitted_bytes,
            }
        });
        let canonical =
            crate::canonical::canonical_envelope_bytes(&request_material, u64::MAX, "request")
                .map_err(|_| {
                    MappingRequestError::new(
                        MappingRequestErrorCode::AllocationFailed,
                        "request",
                        "canonical request allocation failed",
                    )
                })?;
        let request_bytes = u64::try_from(canonical.len()).map_err(|_| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "request",
                "canonical request byte count exceeds the supported integer range",
            )
        })?;
        check_limit(
            request_bytes,
            limits.maximum_request_bytes,
            MappingRequestErrorCode::RequestLimitExceeded,
            "request",
        )?;

        Ok(Self {
            package: package.clone(),
            source_package,
            obligations,
            native,
            model,
            semantic,
            profile,
            limits,
            request_bytes,
            expression_nodes,
            nesting_depth,
        })
    }

    /// Strict bound package retained by this request.
    pub fn package(&self) -> &BoundPackage {
        &self.package
    }

    /// Exact source-package reference and recomputed digest.
    pub fn source_package(&self) -> &MappingSourcePackageRef {
        &self.source_package
    }

    /// Admitted obligations in authored source order.
    pub fn obligations(&self) -> &[AdmittedMappingObligation] {
        &self.obligations
    }

    /// Exact native-source selection.
    pub fn native_selection(&self) -> &NativeSourceSelection {
        &self.native
    }

    /// Exact authoritative-model selection.
    pub fn model_selection(&self) -> &ModelSourceSelection {
        &self.model
    }

    /// Exact semantic-profile selection.
    pub fn semantic_selection(&self) -> &SemanticSourceSelection {
        &self.semantic
    }

    /// Exact target profile.
    pub fn profile(&self) -> &OutputMappingProfile {
        &self.profile
    }

    /// Complete admitted aggregate limits.
    pub fn limits(&self) -> &MappingLimits {
        &self.limits
    }

    /// Canonical semantic request-material bytes charged at admission.
    pub const fn request_bytes(&self) -> u64 {
        self.request_bytes
    }

    /// Aggregate checked-expression node count.
    pub const fn expression_nodes(&self) -> u64 {
        self.expression_nodes
    }

    /// Maximum checked-expression nesting depth.
    pub const fn nesting_depth(&self) -> u64 {
        self.nesting_depth
    }
}

/// Invoke one exact-profile mapper once per obligation and expose records only
/// after the complete population and all aggregate limits validate.
pub fn map_admitted_request<M: OutputMapper>(
    request: &AdmittedMappingRequest,
    mapper: &mut M,
    cancellation: MappingCancellation,
) -> Result<CompletedMappings, MappingRequestError> {
    if cancellation == MappingCancellation::Cancelled {
        return Err(MappingRequestError::new(
            MappingRequestErrorCode::Cancelled,
            "mapping",
            "mapping was cancelled before target dispatch",
        ));
    }
    if mapper.profile() != request.profile() {
        return Err(MappingRequestError::new(
            MappingRequestErrorCode::TargetProfileMismatch,
            "mapper.profile",
            "mapper profile does not equal the admitted request profile",
        ));
    }

    let count = request.obligations().len();
    let mut records = Vec::new();
    records.try_reserve_exact(count).map_err(|_| {
        MappingRequestError::new(
            MappingRequestErrorCode::AllocationFailed,
            "mapping.records",
            "mapping record allocation failed",
        )
    })?;
    let mut fragments = Vec::new();
    fragments.try_reserve_exact(count).map_err(|_| {
        MappingRequestError::new(
            MappingRequestErrorCode::AllocationFailed,
            "mapping.fragments",
            "mapping fragment allocation failed",
        )
    })?;
    let mut mapping_work = 0_u64;
    let mut emitted_bytes = 0_u64;

    for obligation in request.obligations() {
        let remaining = request
            .limits()
            .maximum_mapping_work()
            .checked_sub(mapping_work)
            .ok_or_else(|| {
                MappingRequestError::new(
                    MappingRequestErrorCode::ArithmeticOverflow,
                    "mapping.work",
                    "mapping work accounting exceeded the admitted maximum",
                )
            })?;
        if remaining == 0 {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::MappingWorkLimitExceeded,
                "mapping.work",
                "no mapping work remains for the next admitted obligation",
            ));
        }
        let candidate = mapper.map_obligation(obligation, MappingWorkBudget { remaining })?;
        if candidate.obligation() != obligation.identity() {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::CandidateObligationMismatch,
                "candidate.obligation",
                "mapper candidate names a different source obligation",
            ));
        }
        if candidate.source_state() != obligation.source_state() {
            return Err(MappingRequestError::new(
                MappingRequestErrorCode::CandidateSourceStateMismatch,
                "candidate.source_state",
                "mapper candidate changed the retained source assessment state",
            ));
        }
        mapping_work = mapping_work.checked_add(candidate.work()).ok_or_else(|| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "mapping.work",
                "aggregate mapping work overflowed",
            )
        })?;
        check_limit(
            mapping_work,
            request.limits().maximum_mapping_work(),
            MappingRequestErrorCode::MappingWorkLimitExceeded,
            "mapping.work",
        )?;
        let fragment_bytes = u64::try_from(candidate.fragment().len()).map_err(|_| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "mapping.emitted_bytes",
                "fragment byte count exceeds the supported integer range",
            )
        })?;
        let next_emitted = emitted_bytes.checked_add(fragment_bytes).ok_or_else(|| {
            MappingRequestError::new(
                MappingRequestErrorCode::ArithmeticOverflow,
                "mapping.emitted_bytes",
                "aggregate emitted byte count overflowed",
            )
        })?;
        check_limit(
            next_emitted,
            request.limits().maximum_emitted_bytes(),
            MappingRequestErrorCode::EmittedBytesLimitExceeded,
            "mapping.emitted_bytes",
        )?;
        let mut absolute_regions = Vec::new();
        absolute_regions
            .try_reserve_exact(candidate.local_regions().len())
            .map_err(|_| {
                MappingRequestError::new(
                    MappingRequestErrorCode::AllocationFailed,
                    "record.output_regions",
                    "absolute region allocation failed",
                )
            })?;
        for region in candidate.local_regions() {
            let absolute = region.shifted(emitted_bytes)?;
            if absolute.end() > next_emitted {
                return Err(MappingRequestError::new(
                    MappingRequestErrorCode::InvalidOutputRegion,
                    "record.output_regions",
                    "absolute output region exceeds the accumulated target bytes",
                ));
            }
            absolute_regions.push(absolute);
        }
        let record = build_mapping_record(request, obligation, &candidate, absolute_regions)?;
        fragments.push(candidate.fragment);
        records.push(record);
        emitted_bytes = next_emitted;
    }

    Ok(CompletedMappings {
        request: request.clone(),
        records,
        fragments,
        mapping_work,
        emitted_bytes,
    })
}

#[derive(Serialize)]
struct MappingRecordIdentityMaterial<'a> {
    identity_version: &'static str,
    source: &'a MappingRecordSource,
    source_state: SourceFactState,
    dependencies: &'a [MappingDependencyRef],
    target_profile: &'a OutputMappingProfile,
    disposition: MappingDisposition,
    conditions: &'a [MappingCondition],
    causes: &'a [MappingCause],
    output_regions: &'a [OutputByteRegion],
    #[serde(skip_serializing_if = "Option::is_none")]
    observation_adequacy: Option<&'a ObservationAdequacyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_adequacy: Option<&'a ProtocolAdequacyRef>,
}

fn build_mapping_record(
    request: &AdmittedMappingRequest,
    obligation: &AdmittedMappingObligation,
    candidate: &MappingCandidate,
    output_regions: Vec<OutputByteRegion>,
) -> Result<OutputMappingRecord, MappingRequestError> {
    let source = MappingRecordSource::from_clause(obligation.clause());
    let material = MappingRecordIdentityMaterial {
        identity_version: OUTPUT_MAPPING_RECORD_IDENTITY_VERSION,
        source: &source,
        source_state: candidate.source_state,
        dependencies: &candidate.dependencies,
        target_profile: request.profile(),
        disposition: candidate.disposition,
        conditions: &candidate.conditions,
        causes: &candidate.causes,
        output_regions: &output_regions,
        observation_adequacy: candidate.observation_adequacy.as_ref(),
        protocol_adequacy: candidate.protocol_adequacy.as_ref(),
    };
    let value = serde_json::to_value(material).map_err(|_| {
        MappingRequestError::new(
            MappingRequestErrorCode::AllocationFailed,
            "record.identity",
            "mapping record identity material allocation failed",
        )
    })?;
    let canonical = crate::canonical::canonical_envelope_bytes(&value, u64::MAX, "record.identity")
        .map_err(|_| {
            MappingRequestError::new(
                MappingRequestErrorCode::AllocationFailed,
                "record.identity",
                "mapping record canonicalization failed",
            )
        })?;
    Ok(OutputMappingRecord {
        record_id: MappingRecordId::digest(&canonical),
        source,
        source_state: candidate.source_state,
        dependencies: candidate.dependencies.clone(),
        target_profile: request.profile().clone(),
        disposition: candidate.disposition,
        conditions: candidate.conditions.clone(),
        causes: candidate.causes.clone(),
        output_regions,
        observation_adequacy: candidate.observation_adequacy.clone(),
        protocol_adequacy: candidate.protocol_adequacy.clone(),
    })
}

fn check_limit(
    actual: u64,
    maximum: u64,
    code: MappingRequestErrorCode,
    path: &'static str,
) -> Result<(), MappingRequestError> {
    if actual > maximum {
        Err(MappingRequestError::new(
            code,
            path,
            "output-mapping aggregate exceeds its admitted limit",
        ))
    } else {
        Ok(())
    }
}

fn validate_qualified_members(
    owner: &str,
    identity: &str,
    revision: &str,
    path: &'static str,
) -> Result<(), MappingRequestError> {
    if [owner, identity, revision]
        .into_iter()
        .all(valid_selection_member)
    {
        Ok(())
    } else {
        Err(MappingRequestError::new(
            MappingRequestErrorCode::InvalidQualifiedReference,
            path,
            "qualified owner, identity, and revision must be nonempty bounded visible ASCII",
        ))
    }
}

fn classify_unknown(package: &BoundPackage, identity: &ClauseRef) -> MappingRequestError {
    let code = if identity.requirement().package() != package.package().id() {
        MappingRequestErrorCode::ForeignObligation
    } else if package.package().requirements().iter().any(|requirement| {
        requirement.id() == identity.requirement().requirement()
            && requirement.revision() != identity.requirement().revision()
    }) {
        MappingRequestErrorCode::StaleObligation
    } else {
        MappingRequestErrorCode::UnknownObligation
    };
    MappingRequestError::new(
        code,
        "request.obligations",
        "obligation does not resolve to an executable clause in the bound package",
    )
}
