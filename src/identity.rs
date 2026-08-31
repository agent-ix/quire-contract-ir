use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Box<SourceSpan>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<SemanticIdentity>,
}

impl Diagnostic {
    fn error(code: DiagnosticCode, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            path: path.into(),
            span: None,
            related: Vec::new(),
        }
    }

    fn at_span(mut self, span: &SourceSpan) -> Self {
        self.span = Some(Box::new(span.clone()));
        self
    }

    fn related_to(mut self, identity: SemanticIdentity) -> Self {
        self.related.push(identity);
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
                path,
            })
        }
    }

    pub fn requirement(&self) -> &RequirementRef {
        &self.requirement
    }

    pub const fn kind(&self) -> DependencyKind {
        self.kind
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
            path: Vec<DependencyName>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.requirement, wire.kind, wire.path).map_err(D::Error::custom)
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub fn from_json_str(input: &str) -> Result<Self, Vec<Diagnostic>> {
        let wire: WirePackage = serde_json::from_str(input).map_err(|error| {
            vec![Diagnostic::error(
                DiagnosticCode::InvalidWireFormat,
                error.to_string(),
                "document",
            )]
        })?;
        wire.validate()
    }
}

#[derive(Deserialize)]
struct WirePackage {
    id: String,
    schema_version: WireSchemaVersion,
    source: WireSourceIdentity,
    requirements: Vec<WireRequirement>,
}

impl WirePackage {
    fn validate(self) -> Result<ContractPackage<ReferenceBody>, Vec<Diagnostic>> {
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
}

#[derive(Deserialize)]
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
struct WireDependencyIdentity {
    requirement: WireRequirementRef,
    kind: DependencyKind,
    path: Vec<String>,
}

impl WireDependencyIdentity {
    fn validate(self) -> Result<DependencyIdentity, Diagnostic> {
        let path = self
            .path
            .into_iter()
            .map(DependencyName::new)
            .collect::<Result<Vec<_>, _>>()?;
        DependencyIdentity::new(self.requirement.validate()?, self.kind, path)
    }
}

#[derive(Deserialize)]
#[serde(tag = "node", rename_all = "snake_case")]
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
#[serde(tag = "kind", rename_all = "snake_case")]
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
