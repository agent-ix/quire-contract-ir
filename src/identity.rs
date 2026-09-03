use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{
    de::{DeserializeOwned, Error as _},
    ser::{Error as _, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

const IDENTIFIER_RULE: &str =
    "must start with an ASCII letter and contain only ASCII letters, digits, '.', '_', or '-'";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
}

macro_rules! diagnostic_codes {
    ($( $variant:ident => $wire:literal ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        pub enum DiagnosticCode {
            $( #[serde(rename = $wire)] $variant, )+
        }

        impl DiagnosticCode {
            pub const ALL: &'static [Self] = &[$( Self::$variant, )+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $wire, )+
                }
            }
        }
    };
}

diagnostic_codes! {
    InvalidPackageNamespace => "invalid_package_namespace",
    InvalidWireFormat => "invalid_wire_format",
    InvalidSchemaVersion => "invalid_schema_version",
    InvalidIdentifier => "invalid_identifier",
    InvalidRequirementRevision => "invalid_requirement_revision",
    InvalidSourceRevision => "invalid_source_revision",
    DuplicateRequirement => "duplicate_requirement",
    DuplicateClause => "duplicate_clause",
    CrossPackageReference => "cross_package_reference",
    InvalidSourceSpan => "invalid_source_span",
    FloatingExecutableClause => "floating_executable_clause",
    InformationalClauseAnchored => "informational_clause_anchored",
    IncompatibleClauseAnchor => "incompatible_clause_anchor",
    MalformedReference => "malformed_reference",
    StaleRequirementRevision => "stale_requirement_revision",
    OrphanedRequirementReference => "orphaned_requirement_reference",
    OrphanedClauseReference => "orphaned_clause_reference",
    DuplicateTypeDeclaration => "duplicate_type_declaration",
    DuplicateValueDeclaration => "duplicate_value_declaration",
    DuplicateFunctionDeclaration => "duplicate_function_declaration",
    DuplicateField => "duplicate_field",
    DuplicateVariant => "duplicate_variant",
    DuplicateParameter => "duplicate_parameter",
    EmptyEnum => "empty_enum",
    InvalidNumericBounds => "invalid_numeric_bounds",
    TextBoundExceeded => "text_bound_exceeded",
    UnboundedCollection => "unbounded_collection",
    CollectionBoundExceeded => "collection_bound_exceeded",
    OrphanedTypeReference => "orphaned_type_reference",
    RecursiveType => "recursive_type",
    OrphanedValueReference => "orphaned_value_reference",
    OrphanedFunctionReference => "orphaned_function_reference",
    InvalidStateObservation => "invalid_state_observation",
    InvalidScope => "invalid_scope",
    ArityMismatch => "arity_mismatch",
    IllTypedExpression => "ill_typed_expression",
    ResultTypeMismatch => "result_type_mismatch",
    NonBooleanClauseRoot => "non_boolean_clause_root",
    PotentiallyUndefined => "potentially_undefined",
    ExpressionTooLarge => "expression_too_large",
    UnsupportedSchemaVersion => "unsupported_schema_version",
    UnregisteredMigration => "unregistered_migration",
    CanonicalizationResourceExhausted => "canonicalization_resource_exhausted",
    DuplicateArtifactTrace => "duplicate_artifact_trace",
    StaleTraceDigest => "stale_trace_digest",
    SemanticInputTooLarge => "semantic_input_too_large",
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinednessObligationKind {
    OptionPresence,
    NonZeroDivisor,
    IndexInBounds,
    CheckedRange,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateObservation {
    Current,
    Pre,
    Post,
}

impl<'de> Deserialize<'de> for DiagnosticCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::ALL
            .iter()
            .copied()
            .find(|code| code.as_str() == value)
            .ok_or_else(|| D::Error::custom(format!("unknown diagnostic code {value:?}")))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub path: String,
    pub span: Option<Box<SourceSpan>>,
    pub related: Vec<SemanticIdentity>,
    pub obligation_kind: Option<DefinednessObligationKind>,
}

#[derive(Deserialize)]
struct DiagnosticWire {
    code: DiagnosticCode,
    severity: Severity,
    message: String,
    path: String,
    #[serde(default)]
    span: Option<Box<SourceSpan>>,
    #[serde(default)]
    related: Vec<SemanticIdentity>,
    #[serde(default)]
    obligation_kind: Option<DefinednessObligationKind>,
}

impl Serialize for Diagnostic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !self.has_valid_obligation_kind() {
            return Err(S::Error::custom(
                "diagnostic obligation_kind must be present if and only if code is potentially_undefined",
            ));
        }
        let mut state = serializer.serialize_struct(
            "Diagnostic",
            4 + usize::from(self.span.is_some())
                + usize::from(!self.related.is_empty())
                + usize::from(self.obligation_kind.is_some()),
        )?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("severity", &self.severity)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("path", &self.path)?;
        if let Some(span) = &self.span {
            state.serialize_field("span", span)?;
        }
        if !self.related.is_empty() {
            state.serialize_field("related", &self.related)?;
        }
        if let Some(kind) = self.obligation_kind {
            state.serialize_field("obligation_kind", &kind)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for Diagnostic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DiagnosticWire::deserialize(deserializer)?;
        let diagnostic = Self {
            code: wire.code,
            severity: wire.severity,
            message: wire.message,
            path: wire.path,
            span: wire.span,
            related: wire.related,
            obligation_kind: wire.obligation_kind,
        };
        if diagnostic.has_valid_obligation_kind() {
            Ok(diagnostic)
        } else {
            Err(D::Error::custom(
                "diagnostic obligation_kind must be present if and only if code is potentially_undefined",
            ))
        }
    }
}

impl Diagnostic {
    fn has_valid_obligation_kind(&self) -> bool {
        (self.code == DiagnosticCode::PotentiallyUndefined) == self.obligation_kind.is_some()
    }

    pub(crate) fn error(
        code: DiagnosticCode,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            path: path.into(),
            span: None,
            related: Vec::new(),
            obligation_kind: None,
        }
    }

    pub(crate) fn at_span(mut self, span: &SourceSpan) -> Self {
        self.span = Some(Box::new(span.clone()));
        self
    }

    pub(crate) fn related_to(mut self, identity: SemanticIdentity) -> Self {
        self.related.push(identity);
        self
    }

    pub(crate) fn with_obligation(mut self, kind: DefinednessObligationKind) -> Self {
        self.obligation_kind = Some(kind);
        self
    }
}

impl fmt::Display for Diagnostic {
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

macro_rules! identifier_type {
    ($name:ident, $path:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, Diagnostic> {
                let value = value.into();
                if valid_identifier(&value) {
                    Ok(Self(value))
                } else {
                    Err(Diagnostic::error(
                        DiagnosticCode::InvalidIdentifier,
                        format!("identifier {value:?} {IDENTIFIER_RULE}"),
                        $path,
                    ))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }
    };
}

identifier_type!(SourceDocumentId, "source.document");
identifier_type!(RequirementId, "requirement.id");
identifier_type!(ClauseId, "clause.id");
identifier_type!(AnchorName, "clause.anchor.name");
identifier_type!(DependencyName, "dependency.path");

fn valid_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_alphabetic())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct PackageId(String);

impl PackageId {
    pub fn new(value: impl Into<String>) -> Result<Self, Diagnostic> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.split('/').all(|segment| {
                !segment.is_empty()
                    && segment != "."
                    && segment != ".."
                    && segment
                        .chars()
                        .next()
                        .is_some_and(|first| first.is_ascii_alphanumeric())
                    && segment.chars().all(|character| {
                        character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
                    })
            });
        if valid {
            Ok(Self(value))
        } else {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidPackageNamespace,
                format!("invalid package namespace {value:?}"),
                "package",
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PackageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SchemaVersion {
    major: u16,
    minor: u16,
}

impl SchemaVersion {
    pub const V1_0: Self = Self { major: 1, minor: 0 };
    pub const V1_1: Self = Self { major: 1, minor: 1 };

    pub fn new(major: u16, minor: u16) -> Result<Self, Diagnostic> {
        if major == 0 {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidSchemaVersion,
                "schema major must be positive",
                "schema_version.major",
            ))
        } else {
            Ok(Self { major, minor })
        }
    }

    pub const fn major(self) -> u16 {
        self.major
    }

    pub const fn minor(self) -> u16 {
        self.minor
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            major: u16,
            minor: u16,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.major, wire.minor).map_err(D::Error::custom)
    }
}

macro_rules! positive_revision {
    ($name:ident, $code:expr, $path:literal) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            pub fn new(value: u64) -> Result<Self, Diagnostic> {
                if value == 0 {
                    Err(Diagnostic::error($code, "revision must be positive", $path))
                } else {
                    Ok(Self(value))
                }
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::new(u64::deserialize(deserializer)?).map_err(D::Error::custom)
            }
        }
    };
}

positive_revision!(
    SourceRevision,
    DiagnosticCode::InvalidSourceRevision,
    "source.revision"
);
positive_revision!(
    RequirementRevision,
    DiagnosticCode::InvalidRequirementRevision,
    "requirement.revision"
);

impl RequirementRevision {
    pub fn advance(self, next: u64) -> Result<Self, Diagnostic> {
        if next <= self.0 {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidRequirementRevision,
                format!("revision {next} must be greater than {}", self.0),
                "requirement.revision",
            ))
        } else {
            Self::new(next)
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SourceIdentity {
    document: SourceDocumentId,
    revision: SourceRevision,
}

impl SourceIdentity {
    pub fn new(document: SourceDocumentId, revision: SourceRevision) -> Self {
        Self { document, revision }
    }

    pub fn document(&self) -> &SourceDocumentId {
        &self.document
    }

    pub const fn revision(&self) -> SourceRevision {
        self.revision
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SourceLocation {
    source: SourceIdentity,
    line: u32,
    column: u32,
    byte_offset: u64,
}

impl<'de> Deserialize<'de> for SourceLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            source: SourceIdentity,
            line: u32,
            column: u32,
            byte_offset: u64,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.source, wire.line, wire.column, wire.byte_offset).map_err(D::Error::custom)
    }
}

impl SourceLocation {
    pub fn new(
        source: SourceIdentity,
        line: u32,
        column: u32,
        byte_offset: u64,
    ) -> Result<Self, Diagnostic> {
        if line == 0 || column == 0 {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidSourceSpan,
                "source lines and columns are one-based",
                "source_span",
            ))
        } else {
            Ok(Self {
                source,
                line,
                column,
                byte_offset,
            })
        }
    }

    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }

    pub const fn line(&self) -> u32 {
        self.line
    }

    pub const fn column(&self) -> u32 {
        self.column
    }

    pub const fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SourceSpan {
    start: SourceLocation,
    end: SourceLocation,
}

impl SourceSpan {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Result<Self, Diagnostic> {
        let positions_increase = start.line > 0
            && start.column > 0
            && end.line > 0
            && end.column > 0
            && (start.line, start.column, start.byte_offset)
                <= (end.line, end.column, end.byte_offset)
            && start.byte_offset <= end.byte_offset;
        if start.source != end.source || !positions_increase {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidSourceSpan,
                "span endpoints must share a source and increase monotonically",
                "source_span",
            ))
        } else {
            Ok(Self { start, end })
        }
    }

    pub fn source(&self) -> &SourceIdentity {
        &self.start.source
    }

    pub fn start(&self) -> &SourceLocation {
        &self.start
    }

    pub fn end(&self) -> &SourceLocation {
        &self.end
    }
}

impl<'de> Deserialize<'de> for SourceSpan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            start: SourceLocation,
            end: SourceLocation,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.start, wire.end).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RequirementRef {
    package: PackageId,
    requirement: RequirementId,
    revision: RequirementRevision,
}

impl RequirementRef {
    pub fn new(
        package: PackageId,
        requirement: RequirementId,
        revision: RequirementRevision,
    ) -> Self {
        Self {
            package,
            requirement,
            revision,
        }
    }

    pub fn parse(package: &str, requirement: &str, revision: u64) -> Result<Self, Diagnostic> {
        let package = PackageId::new(package).map_err(|mut diagnostic| {
            diagnostic.path = "reference.package".to_owned();
            diagnostic
        })?;
        let requirement = RequirementId::new(requirement).map_err(|mut diagnostic| {
            diagnostic.path = "reference.requirement".to_owned();
            diagnostic
        })?;
        let revision = RequirementRevision::new(revision).map_err(|mut diagnostic| {
            diagnostic.path = "reference.revision".to_owned();
            diagnostic
        })?;
        Ok(Self {
            package,
            requirement,
            revision,
        })
    }

    pub fn advance(&self, next: u64) -> Result<Self, Diagnostic> {
        Ok(Self {
            package: self.package.clone(),
            requirement: self.requirement.clone(),
            revision: self.revision.advance(next)?,
        })
    }

    pub fn package(&self) -> &PackageId {
        &self.package
    }

    pub fn requirement(&self) -> &RequirementId {
        &self.requirement
    }

    pub const fn revision(&self) -> RequirementRevision {
        self.revision
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ClauseRef {
    requirement: RequirementRef,
    clause: ClauseId,
}

impl ClauseRef {
    pub fn new(requirement: RequirementRef, clause: ClauseId) -> Self {
        Self {
            requirement,
            clause,
        }
    }

    pub fn requirement(&self) -> &RequirementRef {
        &self.requirement
    }

    pub fn clause(&self) -> &ClauseId {
        &self.clause
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SemanticIdentity {
    Package { package: PackageId },
    Requirement { reference: RequirementRef },
    Clause { reference: ClauseRef },
    Dependency { identity: DependencyIdentity },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Input,
    State,
    Field,
    EnumVariant,
    PureFunction,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DependencyIdentity {
    requirement: RequirementRef,
    kind: DependencyKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    observation: Option<StateObservation>,
    path: Vec<DependencyName>,
}

impl DependencyIdentity {
    pub fn new(
        requirement: RequirementRef,
        kind: DependencyKind,
        path: Vec<DependencyName>,
    ) -> Result<Self, Diagnostic> {
        if path.is_empty() {
            Err(Diagnostic::error(
                DiagnosticCode::MalformedReference,
                "dependency path must not be empty",
                "dependency.path",
            ))
        } else {
            Ok(Self {
                requirement,
                kind,
                observation: None,
                path,
            })
        }
    }

    pub fn new_observed(
        requirement: RequirementRef,
        kind: DependencyKind,
        observation: StateObservation,
        path: Vec<DependencyName>,
    ) -> Result<Self, Diagnostic> {
        let mut identity = Self::new(requirement, kind, path)?;
        identity.observation = Some(observation);
        Ok(identity)
    }

    pub fn requirement(&self) -> &RequirementRef {
        &self.requirement
    }

    pub const fn kind(&self) -> DependencyKind {
        self.kind
    }

    pub const fn observation(&self) -> Option<StateObservation> {
        self.observation
    }

    pub fn path(&self) -> &[DependencyName] {
        &self.path
    }
}

impl<'de> Deserialize<'de> for DependencyIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            requirement: RequirementRef,
            kind: DependencyKind,
            #[serde(default)]
            observation: Option<StateObservation>,
            path: Vec<DependencyName>,
        }
        let wire = Wire::deserialize(deserializer)?;
        match wire.observation {
            Some(observation) => {
                Self::new_observed(wire.requirement, wire.kind, observation, wire.path)
            }
            None => Self::new(wire.requirement, wire.kind, wire.path),
        }
        .map_err(D::Error::custom)
    }
}

pub trait DependencySource {
    fn visit_dependencies(&self, visitor: &mut dyn FnMut(&DependencyIdentity));

    fn dependencies(&self) -> BTreeSet<DependencyIdentity> {
        let mut dependencies = BTreeSet::new();
        self.visit_dependencies(&mut |identity| {
            dependencies.insert(identity.clone());
        });
        dependencies
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "node", rename_all = "snake_case")]
pub enum ReferenceBody {
    Literal,
    Reference { identity: DependencyIdentity },
    Composite { children: Vec<ReferenceBody> },
}

impl DependencySource for ReferenceBody {
    fn visit_dependencies(&self, visitor: &mut dyn FnMut(&DependencyIdentity)) {
        match self {
            Self::Literal => {}
            Self::Reference { identity } => visitor(identity),
            Self::Composite { children } => {
                for child in children {
                    child.visit_dependencies(visitor);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExecutionPoint {
    Initialization { name: AnchorName },
    Handler { name: AnchorName },
    Pre { operation: AnchorName },
    Post { operation: AnchorName },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClauseKind {
    Precondition,
    Postcondition,
    Invariant,
    Assertion,
    Case,
    Information,
}

impl ClauseKind {
    pub const fn executable(self) -> bool {
        !matches!(self, Self::Information)
    }

    const fn accepts(self, anchor: &ExecutionPoint) -> bool {
        match self {
            Self::Precondition => matches!(anchor, ExecutionPoint::Pre { .. }),
            Self::Postcondition => matches!(anchor, ExecutionPoint::Post { .. }),
            Self::Invariant => matches!(
                anchor,
                ExecutionPoint::Initialization { .. } | ExecutionPoint::Handler { .. }
            ),
            Self::Assertion => true,
            Self::Case => matches!(anchor, ExecutionPoint::Handler { .. }),
            Self::Information => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Clause<B> {
    id: ClauseId,
    kind: ClauseKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor: Option<ExecutionPoint>,
    source: SourceSpan,
    body: B,
}

impl<B> Clause<B> {
    pub fn new(
        id: ClauseId,
        kind: ClauseKind,
        anchor: Option<ExecutionPoint>,
        source: SourceSpan,
        body: B,
    ) -> Result<Self, Diagnostic> {
        match (&anchor, kind) {
            (None, clause_kind) if clause_kind.executable() => {
                return Err(Diagnostic::error(
                    DiagnosticCode::FloatingExecutableClause,
                    "executable clause requires a lowering anchor",
                    "clause.anchor",
                )
                .at_span(&source));
            }
            (Some(_), ClauseKind::Information) => {
                return Err(Diagnostic::error(
                    DiagnosticCode::InformationalClauseAnchored,
                    "informational clause must remain unanchored",
                    "clause.anchor",
                )
                .at_span(&source));
            }
            (Some(execution_point), clause_kind) if !clause_kind.accepts(execution_point) => {
                return Err(Diagnostic::error(
                    DiagnosticCode::IncompatibleClauseAnchor,
                    "clause kind is incompatible with its execution point",
                    "clause.anchor",
                )
                .at_span(&source));
            }
            _ => {}
        }
        Ok(Self {
            id,
            kind,
            anchor,
            source,
            body,
        })
    }

    pub fn id(&self) -> &ClauseId {
        &self.id
    }

    pub const fn kind(&self) -> ClauseKind {
        self.kind
    }

    pub fn anchor(&self) -> Option<&ExecutionPoint> {
        self.anchor.as_ref()
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }

    pub fn body(&self) -> &B {
        &self.body
    }
}

impl<B: DependencySource> Clause<B> {
    pub fn dependencies(&self) -> BTreeSet<DependencyIdentity> {
        self.body.dependencies()
    }
}

impl<'de, B> Deserialize<'de> for Clause<B>
where
    B: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire<B> {
            id: ClauseId,
            kind: ClauseKind,
            anchor: Option<ExecutionPoint>,
            source: SourceSpan,
            body: B,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.id, wire.kind, wire.anchor, wire.source, wire.body).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Requirement<B> {
    id: RequirementId,
    revision: RequirementRevision,
    source: SourceSpan,
    clauses: Vec<Clause<B>>,
}

impl<B> Requirement<B> {
    pub fn new(
        package: &PackageId,
        id: RequirementId,
        revision: RequirementRevision,
        source: SourceSpan,
        clauses: Vec<Clause<B>>,
    ) -> Result<Self, Diagnostic> {
        if clauses.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
            return Err(semantic_input_too_large("requirement.clauses"));
        }
        let mut clause_ids = BTreeMap::new();
        for clause in &clauses {
            if let Some(first_clause) = clause_ids.insert(clause.id.clone(), clause) {
                return Err(Diagnostic::error(
                    DiagnosticCode::DuplicateClause,
                    format!("duplicate clause identifier {}", clause.id),
                    "requirement.clauses",
                )
                .at_span(&clause.source)
                .related_to(SemanticIdentity::Clause {
                    reference: ClauseRef::new(
                        RequirementRef::new(package.clone(), id.clone(), revision),
                        first_clause.id.clone(),
                    ),
                }));
            }
        }
        Ok(Self {
            id,
            revision,
            source,
            clauses,
        })
    }

    pub fn id(&self) -> &RequirementId {
        &self.id
    }

    pub const fn revision(&self) -> RequirementRevision {
        self.revision
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }

    pub fn clauses(&self) -> &[Clause<B>] {
        &self.clauses
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ContractPackage<B> {
    id: PackageId,
    schema_version: SchemaVersion,
    source: SourceIdentity,
    requirements: Vec<Requirement<B>>,
}

impl<B: DependencySource> ContractPackage<B> {
    pub fn new(
        id: PackageId,
        schema_version: SchemaVersion,
        source: SourceIdentity,
        requirements: Vec<Requirement<B>>,
    ) -> Result<Self, Vec<Diagnostic>> {
        if requirements.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
            return Err(vec![semantic_input_too_large("requirements")]);
        }
        let package = Self {
            id,
            schema_version,
            source,
            requirements,
        };
        let diagnostics = package.validate();
        if diagnostics.is_empty() {
            Ok(package)
        } else {
            Err(diagnostics)
        }
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut requirements = BTreeMap::new();
        for requirement in &self.requirements {
            if requirement.source.source() != &self.source {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::InvalidSourceSpan,
                        "requirement source differs from package source identity",
                        "requirements.source",
                    )
                    .at_span(&requirement.source),
                );
            }
            if let Some(first_requirement) =
                requirements.insert(requirement.id.clone(), requirement)
            {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateRequirement,
                        format!("duplicate current requirement {}", requirement.id),
                        "requirements",
                    )
                    .related_to(SemanticIdentity::Requirement {
                        reference: RequirementRef {
                            package: self.id.clone(),
                            requirement: first_requirement.id.clone(),
                            revision: first_requirement.revision,
                        },
                    }),
                );
            }
            for clause in &requirement.clauses {
                if clause.source.source() != &self.source {
                    diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::InvalidSourceSpan,
                            "clause source differs from package source identity",
                            "requirements.clauses.source",
                        )
                        .at_span(&clause.source),
                    );
                }
                for dependency in clause.dependencies() {
                    if dependency.requirement.package != self.id {
                        diagnostics.push(
                            Diagnostic::error(
                                DiagnosticCode::CrossPackageReference,
                                "dependency reference names a different package",
                                "requirements.clauses.body",
                            )
                            .at_span(&clause.source),
                        );
                    } else if let Err(diagnostic) =
                        self.resolve_requirement(&dependency.requirement, Some(&clause.source))
                    {
                        diagnostics.push(diagnostic);
                    }
                }
            }
        }
        diagnostics
    }

    pub fn id(&self) -> &PackageId {
        &self.id
    }

    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }

    pub fn requirements(&self) -> &[Requirement<B>] {
        &self.requirements
    }

    pub fn requirement_ref(&self, requirement: &Requirement<B>) -> RequirementRef {
        RequirementRef {
            package: self.id.clone(),
            requirement: requirement.id.clone(),
            revision: requirement.revision,
        }
    }

    pub fn clause_ref(&self, requirement: &Requirement<B>, clause: &Clause<B>) -> ClauseRef {
        ClauseRef {
            requirement: self.requirement_ref(requirement),
            clause: clause.id.clone(),
        }
    }

    pub fn resolve_requirement(
        &self,
        reference: &RequirementRef,
        span: Option<&SourceSpan>,
    ) -> Result<&Requirement<B>, Diagnostic> {
        if reference.package != self.id {
            return Err(with_optional_span(
                Diagnostic::error(
                    DiagnosticCode::CrossPackageReference,
                    "reference names a different package",
                    "reference.package",
                ),
                span,
            ));
        }
        let Some(requirement) = self
            .requirements
            .iter()
            .find(|candidate| candidate.id == reference.requirement)
        else {
            return Err(with_optional_span(
                Diagnostic::error(
                    DiagnosticCode::OrphanedRequirementReference,
                    "referenced requirement does not exist",
                    "reference.requirement",
                ),
                span,
            ));
        };
        if requirement.revision != reference.revision {
            return Err(with_optional_span(
                Diagnostic::error(
                    DiagnosticCode::StaleRequirementRevision,
                    format!(
                        "referenced revision {} differs from current revision {}",
                        reference.revision.get(),
                        requirement.revision.get()
                    ),
                    "reference.revision",
                )
                .related_to(SemanticIdentity::Requirement {
                    reference: self.requirement_ref(requirement),
                }),
                span,
            ));
        }
        Ok(requirement)
    }

    pub fn resolve_clause(
        &self,
        reference: &ClauseRef,
        span: Option<&SourceSpan>,
    ) -> Result<&Clause<B>, Diagnostic> {
        let requirement = self.resolve_requirement(&reference.requirement, span)?;
        requirement
            .clauses
            .iter()
            .find(|candidate| candidate.id == reference.clause)
            .ok_or_else(|| {
                with_optional_span(
                    Diagnostic::error(
                        DiagnosticCode::OrphanedClauseReference,
                        "referenced clause does not exist",
                        "reference.clause",
                    ),
                    span,
                )
            })
    }
}

fn with_optional_span(diagnostic: Diagnostic, span: Option<&SourceSpan>) -> Diagnostic {
    match span {
        Some(span) => diagnostic.at_span(span),
        None => diagnostic,
    }
}

impl ContractPackage<ReferenceBody> {
    /// Parses the issue #6 JSON representation without reducing semantic failures to text.
    pub fn from_json_str(
        input: &str,
        options: crate::ValidationOptions,
    ) -> Result<Self, Vec<Diagnostic>> {
        debug_assert!(options.is_strict());
        if crate::limits::json_nesting_exceeds(input.as_bytes(), crate::MAX_WIRE_JSON_DEPTH) {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::InvalidWireFormat,
                "JSON nesting exceeds decode limit",
                "document.nesting",
            )]);
        }
        let preflight: VersionPreflight = parse_json_stack_safe(input).map_err(|error| {
            vec![Diagnostic::error(
                DiagnosticCode::InvalidWireFormat,
                error.to_string(),
                "document",
            )]
        })?;
        preflight
            .validate()
            .map_err(|diagnostic| vec![diagnostic])?;
        let wire: WirePackage = parse_json_stack_safe(input).map_err(|error| {
            vec![Diagnostic::error(
                DiagnosticCode::InvalidWireFormat,
                error.to_string(),
                "document",
            )]
        })?;
        wire.validate()
    }

    pub fn from_json_bytes(
        input: &[u8],
        options: crate::ValidationOptions,
    ) -> Result<Self, Vec<Diagnostic>> {
        let input = std::str::from_utf8(input).map_err(|_| {
            vec![Diagnostic::error(
                DiagnosticCode::InvalidWireFormat,
                "document is not UTF-8",
                "document",
            )]
        })?;
        Self::from_json_str(input, options)
    }
}

fn parse_json_stack_safe<T: DeserializeOwned>(input: &str) -> Result<T, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    deserializer.disable_recursion_limit();
    T::deserialize(serde_stacker::Deserializer::new(&mut deserializer))
}

#[derive(Deserialize)]
struct VersionPreflight {
    schema_version: WireSchemaVersion,
}

impl VersionPreflight {
    fn validate(self) -> Result<SchemaVersion, Diagnostic> {
        let version = self.schema_version.validate()?;
        match (version.major(), version.minor()) {
            (1, 0 | 1) => Ok(version),
            (1, _) => Err(Diagnostic::error(
                DiagnosticCode::UnregisteredMigration,
                "schema minor has no registered migration",
                "schema_version",
            )),
            (_, _) => Err(Diagnostic::error(
                DiagnosticCode::UnsupportedSchemaVersion,
                "schema major is unsupported",
                "schema_version.major",
            )),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WirePackage {
    id: String,
    schema_version: WireSchemaVersion,
    source: WireSourceIdentity,
    requirements: Vec<WireRequirement>,
}

impl WirePackage {
    fn validate(self) -> Result<ContractPackage<ReferenceBody>, Vec<Diagnostic>> {
        self.preflight()?;
        let id = PackageId::new(self.id).map_err(|diagnostic| vec![diagnostic])?;
        let schema_version = self
            .schema_version
            .validate()
            .map_err(|diagnostic| vec![diagnostic])?;
        let source = self
            .source
            .validate()
            .map_err(|diagnostic| vec![diagnostic])?;
        let requirements = self
            .requirements
            .into_iter()
            .map(|requirement| requirement.validate(&id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|diagnostic| vec![diagnostic])?;
        ContractPackage::new(id, schema_version, source, requirements)
    }

    fn preflight(&self) -> Result<(), Vec<Diagnostic>> {
        if self.requirements.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
            return Err(vec![semantic_input_too_large("requirements")]);
        }
        let mut nodes = u32::try_from(self.requirements.len()).unwrap_or(u32::MAX);
        let mut bodies = Vec::new();
        for requirement in &self.requirements {
            if requirement.clauses.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                return Err(vec![semantic_input_too_large("requirements.clauses")]);
            }
            nodes =
                nodes.saturating_add(u32::try_from(requirement.clauses.len()).unwrap_or(u32::MAX));
            bodies.extend(
                requirement
                    .clauses
                    .iter()
                    .map(|clause| (&clause.body, 1_u32)),
            );
        }
        while let Some((body, depth)) = bodies.pop() {
            if depth > crate::MAX_SEMANTIC_DEPTH {
                return Err(vec![semantic_input_too_large("requirements.clauses.body")]);
            }
            nodes = nodes.saturating_add(1);
            if nodes > crate::MAX_SEMANTIC_NODES {
                return Err(vec![semantic_input_too_large("package")]);
            }
            if let WireReferenceBody::Composite { children } = body {
                if children.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                    return Err(vec![semantic_input_too_large(
                        "requirements.clauses.body.children",
                    )]);
                }
                bodies.extend(children.iter().map(|child| (child, depth + 1)));
            }
        }
        Ok(())
    }
}

fn semantic_input_too_large(path: &'static str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::SemanticInputTooLarge,
        "semantic input exceeds a fixed validation limit",
        path,
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSchemaVersion {
    major: u16,
    minor: u16,
}

impl WireSchemaVersion {
    fn validate(self) -> Result<SchemaVersion, Diagnostic> {
        SchemaVersion::new(self.major, self.minor)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSourceIdentity {
    document: String,
    revision: u64,
}

impl WireSourceIdentity {
    fn validate(self) -> Result<SourceIdentity, Diagnostic> {
        Ok(SourceIdentity::new(
            SourceDocumentId::new(self.document)?,
            SourceRevision::new(self.revision)?,
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSourceLocation {
    source: WireSourceIdentity,
    line: u32,
    column: u32,
    byte_offset: u64,
}

impl WireSourceLocation {
    fn validate(self) -> Result<SourceLocation, Diagnostic> {
        SourceLocation::new(
            self.source.validate()?,
            self.line,
            self.column,
            self.byte_offset,
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSourceSpan {
    start: WireSourceLocation,
    end: WireSourceLocation,
}

impl WireSourceSpan {
    fn validate(self) -> Result<SourceSpan, Diagnostic> {
        SourceSpan::new(self.start.validate()?, self.end.validate()?)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRequirementRef {
    package: String,
    requirement: String,
    revision: u64,
}

impl WireRequirementRef {
    fn validate(self) -> Result<RequirementRef, Diagnostic> {
        RequirementRef::parse(&self.package, &self.requirement, self.revision)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDependencyIdentity {
    requirement: WireRequirementRef,
    kind: DependencyKind,
    #[serde(default)]
    observation: Option<StateObservation>,
    path: Vec<String>,
}

impl WireDependencyIdentity {
    fn validate(self) -> Result<DependencyIdentity, Diagnostic> {
        let path = self
            .path
            .into_iter()
            .map(DependencyName::new)
            .collect::<Result<Vec<_>, _>>()?;
        match self.observation {
            Some(observation) => DependencyIdentity::new_observed(
                self.requirement.validate()?,
                self.kind,
                observation,
                path,
            ),
            None => DependencyIdentity::new(self.requirement.validate()?, self.kind, path),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "node", rename_all = "snake_case", deny_unknown_fields)]
enum WireReferenceBody {
    Literal,
    Reference { identity: WireDependencyIdentity },
    Composite { children: Vec<WireReferenceBody> },
}

impl WireReferenceBody {
    fn validate(self) -> Result<ReferenceBody, Diagnostic> {
        match self {
            Self::Literal => Ok(ReferenceBody::Literal),
            Self::Reference { identity } => Ok(ReferenceBody::Reference {
                identity: identity.validate()?,
            }),
            Self::Composite { children } => Ok(ReferenceBody::Composite {
                children: children
                    .into_iter()
                    .map(Self::validate)
                    .collect::<Result<Vec<_>, _>>()?,
            }),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum WireExecutionPoint {
    Initialization { name: String },
    Handler { name: String },
    Pre { operation: String },
    Post { operation: String },
}

impl WireExecutionPoint {
    fn validate(self) -> Result<ExecutionPoint, Diagnostic> {
        match self {
            Self::Initialization { name } => Ok(ExecutionPoint::Initialization {
                name: AnchorName::new(name)?,
            }),
            Self::Handler { name } => Ok(ExecutionPoint::Handler {
                name: AnchorName::new(name)?,
            }),
            Self::Pre { operation } => Ok(ExecutionPoint::Pre {
                operation: AnchorName::new(operation)?,
            }),
            Self::Post { operation } => Ok(ExecutionPoint::Post {
                operation: AnchorName::new(operation)?,
            }),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireClause {
    id: String,
    kind: ClauseKind,
    anchor: Option<WireExecutionPoint>,
    source: WireSourceSpan,
    body: WireReferenceBody,
}

impl WireClause {
    fn validate(self) -> Result<Clause<ReferenceBody>, Diagnostic> {
        let id = ClauseId::new(self.id)?;
        let anchor = self.anchor.map(WireExecutionPoint::validate).transpose()?;
        let source = self.source.validate()?;
        let body = self
            .body
            .validate()
            .map_err(|diagnostic| diagnostic.at_span(&source))?;
        Clause::new(id, self.kind, anchor, source, body)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRequirement {
    id: String,
    revision: u64,
    source: WireSourceSpan,
    clauses: Vec<WireClause>,
}

impl WireRequirement {
    fn validate(self, package: &PackageId) -> Result<Requirement<ReferenceBody>, Diagnostic> {
        Requirement::new(
            package,
            RequirementId::new(self.id)?,
            RequirementRevision::new(self.revision)?,
            self.source.validate()?,
            self.clauses
                .into_iter()
                .map(WireClause::validate)
                .collect::<Result<Vec<_>, _>>()?,
        )
    }
}
