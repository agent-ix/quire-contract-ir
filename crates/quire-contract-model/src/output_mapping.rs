//! Target-neutral output-mapping admission governed by FR-032 and TC-043.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Serialize, Serializer};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{BoundClause, BoundPackage, CanonicalDigest, ClauseRef, PackageId, SchemaVersion};

/// Identity version for the deterministic mapping-request material.
pub const OUTPUT_MAPPING_REQUEST_IDENTITY_VERSION: &str =
    "quire.output.mapping-request-identity/v1-draft.1";
/// Initial FS06 mapping revision shared by all three target profiles.
pub const OUTPUT_MAPPING_REVISION: &str = "1-draft.1";

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
