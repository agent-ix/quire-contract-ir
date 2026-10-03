//! The canonical bytes and digests of the five closed object kinds (FR-016).
//!
//! Every byte comes from `quire-canonical`: each object kind is an [`Encode`]
//! projection that pushes its members into the encoder, and the digest is
//! SHA-256 over an explicit domain prefix and the bytes the encoder returns.
//! This module holds no encoder of its own and no `serde_json` value; a
//! projection with a fixed depth is a plain walk over the writer, and the
//! types whose depth follows their input (`ValueType`, `Expression`,
//! `ReferenceBody`) are walked from an explicit stack, never by recursion.

use std::{cmp::Ordering, fmt};

use quire_canonical::{Encode, Error, Limits, Sink, Writer};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

use crate::SchemaVersion;
use crate::{
    BooleanOperator, Clause, CollectionType, ComparisonOperator, ContractPackage,
    DeclarationEnvironment, DependencyIdentity, DependencySource, Diagnostic, DiagnosticCode,
    EnumDeclaration, Expression, ExpressionKind, FunctionParameter, IntegerType, NumericOperator,
    PureFunctionDeclaration, QuantifierDomain, QuantifierKind, RationalType, RecordDeclaration,
    RecordLiteralField, ReferenceBody, Requirement, RequirementRef, SourceSpan, StateObservation,
    TypeDeclaration, TypedExpression, ValueDeclaration, ValueType,
};

pub const CANONICAL_PROFILE: &str = "quire.contract.canonical-json/v1";
const DIGEST_DOMAIN: &[u8] = b"quire-contract-ir";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CanonicalProfile {
    #[serde(rename = "quire.contract.canonical-json/v1")]
    V1,
}

impl CanonicalProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => CANONICAL_PROFILE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalKind {
    Package,
    Requirement,
    Clause,
    Declaration,
    Expression,
}

impl CanonicalKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Package => "package",
            Self::Requirement => "requirement",
            Self::Clause => "clause",
            Self::Declaration => "declaration",
            Self::Expression => "expression",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalBytes(Vec<u8>);

impl CanonicalBytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> u64 {
        u64::try_from(self.0.len()).unwrap_or(u64::MAX)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, quire_canonical::FixedShape)]
pub struct CanonicalDigest([u8; 32]);

impl CanonicalDigest {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn parse(value: &str) -> Result<Self, Diagnostic> {
        if value.len() != 64
            || value
                .bytes()
                .any(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(Diagnostic::error(
                DiagnosticCode::InvalidWireFormat,
                "canonical digest must be exactly 64 lowercase hexadecimal characters",
                "digest",
            ));
        }
        let mut bytes = [0_u8; 32];
        for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            bytes[index] = (hex_value(pair[0]) << 4) | hex_value(pair[1]);
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for CanonicalDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in self.0 {
            formatter.write_str(
                std::str::from_utf8(&[HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 0x0f)]])
                    .map_err(|_| fmt::Error)?,
            )?;
        }
        Ok(())
    }
}

impl Serialize for CanonicalDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for CanonicalDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalOutput {
    kind: CanonicalKind,
    bytes: CanonicalBytes,
    digest: CanonicalDigest,
}

impl CanonicalOutput {
    pub const fn kind(&self) -> CanonicalKind {
        self.kind
    }

    pub fn bytes(&self) -> &CanonicalBytes {
        &self.bytes
    }

    pub const fn digest(&self) -> CanonicalDigest {
        self.digest
    }
}

/// A clause body with a canonical encoding: the `Encode` of the body is its
/// canonical projection (a `ReferenceBody` as it is, a `TypedExpression` as its
/// result type and source-free tree).
pub trait CanonicalBody: Clone + DependencySource + Eq + Serialize + Encode {}

impl CanonicalBody for ReferenceBody {}

impl CanonicalBody for TypedExpression {}

impl<B: CanonicalBody> ContractPackage<B> {
    pub fn canonical_package(
        &self,
        profile: CanonicalProfile,
    ) -> Result<CanonicalOutput, Diagnostic> {
        self.canonical_package_with_limit(profile, u64::MAX)
    }

    pub fn canonical_package_with_limit(
        &self,
        profile: CanonicalProfile,
        maximum_bytes: u64,
    ) -> Result<CanonicalOutput, Diagnostic> {
        require_profile(profile)?;
        ensure_supported(self.schema_version())?;
        canonicalize(
            CanonicalKind::Package,
            &PackageProjection(self),
            maximum_bytes,
            "package",
            None,
        )
    }

    pub fn canonical_requirement(
        &self,
        requirement: &Requirement<B>,
        profile: CanonicalProfile,
    ) -> Result<CanonicalOutput, Diagnostic> {
        self.canonical_requirement_with_limit(requirement, profile, u64::MAX)
    }

    pub fn canonical_requirement_with_limit(
        &self,
        requirement: &Requirement<B>,
        profile: CanonicalProfile,
        maximum_bytes: u64,
    ) -> Result<CanonicalOutput, Diagnostic> {
        require_profile(profile)?;
        ensure_supported(self.schema_version())?;
        let requirement = self
            .requirements()
            .iter()
            .find(|candidate| *candidate == requirement)
            .ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticCode::MalformedReference,
                    "requirement is not a current member of this package",
                    "requirement",
                )
            })?;
        canonicalize(
            CanonicalKind::Requirement,
            &RequirementProjection {
                package: self,
                requirement,
            },
            maximum_bytes,
            "requirement",
            Some(requirement.source()),
        )
    }

    pub fn canonical_clause(
        &self,
        requirement: &Requirement<B>,
        clause: &Clause<B>,
        profile: CanonicalProfile,
    ) -> Result<CanonicalOutput, Diagnostic> {
        self.canonical_clause_with_limit(requirement, clause, profile, u64::MAX)
    }

    pub fn canonical_clause_with_limit(
        &self,
        requirement: &Requirement<B>,
        clause: &Clause<B>,
        profile: CanonicalProfile,
        maximum_bytes: u64,
    ) -> Result<CanonicalOutput, Diagnostic> {
        require_profile(profile)?;
        ensure_supported(self.schema_version())?;
        let requirement = self
            .requirements()
            .iter()
            .find(|candidate| *candidate == requirement)
            .ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticCode::MalformedReference,
                    "requirement is not a current member of this package",
                    "clause.requirement",
                )
            })?;
        let clause = requirement
            .clauses()
            .iter()
            .find(|candidate| *candidate == clause)
            .ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticCode::MalformedReference,
                    "clause is not a member of the supplied requirement",
                    "clause",
                )
            })?;
        canonicalize(
            CanonicalKind::Clause,
            &ClauseProjection {
                requirement: self.requirement_ref(requirement),
                clause,
            },
            maximum_bytes,
            "clause",
            Some(clause.source()),
        )
    }
}

impl DeclarationEnvironment {
    pub fn canonical_declaration(
        &self,
        profile: CanonicalProfile,
    ) -> Result<CanonicalOutput, Diagnostic> {
        self.canonical_declaration_with_limit(profile, u64::MAX)
    }

    pub fn canonical_declaration_with_limit(
        &self,
        profile: CanonicalProfile,
        maximum_bytes: u64,
    ) -> Result<CanonicalOutput, Diagnostic> {
        require_profile(profile)?;
        canonicalize(
            CanonicalKind::Declaration,
            self,
            maximum_bytes,
            "declaration",
            None,
        )
    }
}

impl TypedExpression {
    pub fn canonical_expression(
        &self,
        profile: CanonicalProfile,
    ) -> Result<CanonicalOutput, Diagnostic> {
        self.canonical_expression_with_limit(profile, u64::MAX)
    }

    pub fn canonical_expression_with_limit(
        &self,
        profile: CanonicalProfile,
        maximum_bytes: u64,
    ) -> Result<CanonicalOutput, Diagnostic> {
        require_profile(profile)?;
        canonicalize(
            CanonicalKind::Expression,
            self,
            maximum_bytes,
            "expression",
            Some(self.expression().source()),
        )
    }
}

fn ensure_supported(version: SchemaVersion) -> Result<(), Diagnostic> {
    if version == SchemaVersion::V1_1 {
        Ok(())
    } else {
        Err(Diagnostic::error(
            DiagnosticCode::UnsupportedSchemaVersion,
            "schema version is unsupported",
            "schema_version",
        ))
    }
}

fn require_profile(profile: CanonicalProfile) -> Result<(), Diagnostic> {
    match profile {
        CanonicalProfile::V1 => Ok(()),
    }
}

fn unicode_cmp(left: &str, right: &str) -> Ordering {
    left.chars().cmp(right.chars())
}

/// The profile envelope `{"kind", "profile", "value"}` around one projection.
struct Envelope<'a, V: Encode + ?Sized> {
    kind: CanonicalKind,
    value: &'a V,
}

impl<V: Encode + ?Sized> Encode for Envelope<'_, V> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("kind")?;
        writer.string(self.kind.as_str())?;
        writer.name("profile")?;
        writer.string(CANONICAL_PROFILE)?;
        writer.name("value")?;
        self.value.encode_into(writer)?;
        writer.end_object()
    }
}

/// The canonical bytes of `value` under `maximum_bytes` and their digest.
///
/// Any refusal of the encoder is a failed canonicalization step, refused with
/// the one code STD-001 registers for it; see [`encoder_refusal`] for the
/// message that tells a limit from an encoder fault.
fn canonicalize<V: Encode + ?Sized>(
    kind: CanonicalKind,
    value: &V,
    maximum_bytes: u64,
    path: &str,
    span: Option<&SourceSpan>,
) -> Result<CanonicalOutput, Diagnostic> {
    let encoded = quire_canonical::to_vec(&Envelope { kind, value }, Limits::new(maximum_bytes))
        .map_err(|error| encoder_refusal(&error, path, span))?;
    let bytes = CanonicalBytes(encoded);
    let digest = digest(kind, bytes.as_slice());
    Ok(CanonicalOutput {
        kind,
        bytes,
        digest,
    })
}

/// The refusal for an encoder error. STD-001 registers one code for a failed
/// canonicalization step, so every error carries it; the message says which
/// it was. A limit or a failed reservation is the resource condition the code
/// names. Anything else (a protocol violation, an internal fault, an integer
/// past 2^53) cannot come from the model's own types, which bound every
/// number they hold, so it is a bug in a walk and says so rather than posing
/// as exhaustion.
pub(crate) fn encoder_refusal(error: &Error, path: &str, span: Option<&SourceSpan>) -> Diagnostic {
    let message = match error {
        Error::Limit(_) | Error::Allocation { .. } => {
            "canonical byte allocation exceeded available resources".to_owned()
        }
        other => format!("canonical encoder failed unexpectedly: {other}"),
    };
    let diagnostic = Diagnostic::error(
        DiagnosticCode::CanonicalizationResourceExhausted,
        message,
        path,
    );
    match span {
        Some(span) => diagnostic.at_span(span),
        None => diagnostic,
    }
}

/// SHA-256 over the domain prefix `quire-contract-ir`, a zero byte, the
/// profile identity, a zero byte, the kind, a zero byte and the canonical
/// bytes of the envelope (FR-016-AC-8). This is not the encoder crate's own
/// length-prefixed domain digest, which is a different identity.
fn digest(kind: CanonicalKind, bytes: &[u8]) -> CanonicalDigest {
    let mut hasher = Sha256::new();
    hasher.update(DIGEST_DOMAIN);
    hasher.update([0]);
    hasher.update(CANONICAL_PROFILE.as_bytes());
    hasher.update([0]);
    hasher.update(kind.as_str().as_bytes());
    hasher.update([0]);
    hasher.update(bytes);
    CanonicalDigest(hasher.finalize().into())
}

/// The package projection: `{"id", "requirements", "schema_version"}` with the
/// requirements in identifier order.
struct PackageProjection<'a, B>(&'a ContractPackage<B>);

impl<B: CanonicalBody> Encode for PackageProjection<'_, B> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        let package = self.0;
        let mut requirements = package.requirements().iter().collect::<Vec<_>>();
        requirements.sort_by(|left, right| unicode_cmp(left.id().as_str(), right.id().as_str()));
        writer.begin_object()?;
        writer.name("id")?;
        writer.string(package.id().as_str())?;
        writer.name("requirements")?;
        writer.begin_array()?;
        for requirement in requirements {
            RequirementProjection {
                package,
                requirement,
            }
            .encode_into(writer)?;
        }
        writer.end_array()?;
        writer.name("schema_version")?;
        writer.serialize(&package.schema_version())?;
        writer.end_object()
    }
}

/// The requirement projection: `{"clauses", "id", "package", "revision"}` with
/// the clauses in identifier order; the revision is a JSON number.
struct RequirementProjection<'a, B> {
    package: &'a ContractPackage<B>,
    requirement: &'a Requirement<B>,
}

impl<B: CanonicalBody> Encode for RequirementProjection<'_, B> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        let reference = self.package.requirement_ref(self.requirement);
        let mut clauses = self.requirement.clauses().iter().collect::<Vec<_>>();
        clauses.sort_by(|left, right| unicode_cmp(left.id().as_str(), right.id().as_str()));
        writer.begin_object()?;
        writer.name("clauses")?;
        writer.begin_array()?;
        for clause in clauses {
            ClauseProjection {
                requirement: reference.clone(),
                clause,
            }
            .encode_into(writer)?;
        }
        writer.end_array()?;
        writer.name("id")?;
        writer.string(self.requirement.id().as_str())?;
        writer.name("package")?;
        writer.string(self.package.id().as_str())?;
        writer.name("revision")?;
        writer.integer(i128::from(self.requirement.revision().get()))?;
        writer.end_object()
    }
}

/// The clause projection: `{"anchor"?, "body", "id", "kind", "requirement"}`.
/// An absent anchor is omitted, never a `null`.
struct ClauseProjection<'a, B> {
    requirement: RequirementRef,
    clause: &'a Clause<B>,
}

impl<B: CanonicalBody> Encode for ClauseProjection<'_, B> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        if let Some(anchor) = self.clause.anchor() {
            writer.name("anchor")?;
            writer.serialize(anchor)?;
        }
        writer.name("body")?;
        self.clause.body().encode_into(writer)?;
        writer.name("id")?;
        writer.string(self.clause.id().as_str())?;
        writer.name("kind")?;
        writer.serialize(&self.clause.kind())?;
        writer.name("requirement")?;
        writer.serialize(&self.requirement)?;
        writer.end_object()
    }
}

/// The declaration projection: source spans are excluded, the types, values
/// and functions are in name order, and an enum's variants and a record's
/// fields are in name order; a function's parameters keep their sequence.
impl Encode for DeclarationEnvironment {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        let mut types = self.types().iter().collect::<Vec<_>>();
        types.sort_by(|left, right| unicode_cmp(left.name().as_str(), right.name().as_str()));
        let mut values = self.values().iter().collect::<Vec<_>>();
        values.sort_by(|left, right| unicode_cmp(left.name().as_str(), right.name().as_str()));
        let mut functions = self.functions().iter().collect::<Vec<_>>();
        functions.sort_by(|left, right| unicode_cmp(left.name().as_str(), right.name().as_str()));
        writer.begin_object()?;
        writer.name("functions")?;
        writer.begin_array()?;
        for function in functions {
            encode_function(writer, function)?;
        }
        writer.end_array()?;
        writer.name("owner")?;
        writer.serialize(self.owner())?;
        writer.name("types")?;
        writer.begin_array()?;
        for declaration in types {
            encode_type_declaration(writer, declaration)?;
        }
        writer.end_array()?;
        writer.name("values")?;
        writer.begin_array()?;
        for value in values {
            encode_value_declaration(writer, value)?;
        }
        writer.end_array()?;
        writer.end_object()
    }
}

fn encode_function<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    function: &PureFunctionDeclaration,
) -> Result<(), Error> {
    writer.begin_object()?;
    writer.name("name")?;
    writer.string(function.name().as_str())?;
    writer.name("parameters")?;
    writer.begin_array()?;
    for parameter in function.parameters() {
        encode_parameter(writer, parameter)?;
    }
    writer.end_array()?;
    writer.name("result_type")?;
    function.result_type().encode_into(writer)?;
    writer.end_object()
}

fn encode_parameter<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    parameter: &FunctionParameter,
) -> Result<(), Error> {
    writer.begin_object()?;
    writer.name("name")?;
    writer.string(parameter.name().as_str())?;
    writer.name("value_type")?;
    parameter.value_type().encode_into(writer)?;
    writer.end_object()
}

fn encode_value_declaration<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    value: &ValueDeclaration,
) -> Result<(), Error> {
    writer.begin_object()?;
    writer.name("kind")?;
    writer.serialize(&value.kind())?;
    writer.name("name")?;
    writer.string(value.name().as_str())?;
    writer.name("value_type")?;
    value.value_type().encode_into(writer)?;
    writer.end_object()
}

fn encode_type_declaration<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    declaration: &TypeDeclaration,
) -> Result<(), Error> {
    writer.begin_object()?;
    writer.name("declaration")?;
    match declaration {
        TypeDeclaration::Enum { declaration } => {
            encode_enum(writer, declaration)?;
            writer.name("kind")?;
            writer.string("enum")?;
        }
        TypeDeclaration::Record { declaration } => {
            encode_record(writer, declaration)?;
            writer.name("kind")?;
            writer.string("record")?;
        }
    }
    writer.end_object()
}

fn encode_enum<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    declaration: &EnumDeclaration,
) -> Result<(), Error> {
    let mut variants = declaration.variants().iter().collect::<Vec<_>>();
    variants.sort_by(|left, right| unicode_cmp(left.name().as_str(), right.name().as_str()));
    writer.begin_object()?;
    writer.name("name")?;
    writer.string(declaration.name().as_str())?;
    writer.name("variants")?;
    writer.begin_array()?;
    for variant in variants {
        writer.begin_object()?;
        writer.name("name")?;
        writer.string(variant.name().as_str())?;
        writer.end_object()?;
    }
    writer.end_array()?;
    writer.end_object()
}

fn encode_record<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    declaration: &RecordDeclaration,
) -> Result<(), Error> {
    let mut fields = declaration.fields().iter().collect::<Vec<_>>();
    fields.sort_by(|left, right| unicode_cmp(left.name().as_str(), right.name().as_str()));
    writer.begin_object()?;
    writer.name("fields")?;
    writer.begin_array()?;
    for field in fields {
        writer.begin_object()?;
        writer.name("name")?;
        writer.string(field.name().as_str())?;
        writer.name("value_type")?;
        field.value_type().encode_into(writer)?;
        writer.end_object()?;
    }
    writer.end_array()?;
    writer.name("name")?;
    writer.string(declaration.name().as_str())?;
    writer.end_object()
}

/// The expression projection: `{"result_type", "tree"}`, the tree without
/// source spans.
impl Encode for TypedExpression {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("result_type")?;
        self.value_type().encode_into(writer)?;
        writer.name("tree")?;
        self.expression().encode_into(writer)?;
        writer.end_object()
    }
}

/// `{"kind": <node>}`: the expression's node without its source span.
impl Encode for Expression {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        walk(writer, Step::Expression(self))
    }
}

/// `{"kind": ..., ...}` tagged by `kind`, as the type serializes.
impl Encode for ValueType {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        walk(writer, Step::Type(self))
    }
}

/// `{"element": <type>, "maximum_items": <number>}`.
impl Encode for CollectionType {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        walk(writer, Step::Collection(self))
    }
}

/// `{"node": "literal" | "reference" | "composite", ...}`.
impl Encode for ReferenceBody {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        walk(writer, Step::Body(self))
    }
}

/// One pending unit of the explicit-stack walk. A node that has children
/// pushes its own members back as steps, so the depth of the data costs heap
/// in the stack and never native stack.
enum Step<'a> {
    Type(&'a ValueType),
    Collection(&'a CollectionType),
    Expression(&'a Expression),
    Node(&'a ExpressionKind),
    Items(&'a [Expression]),
    Fields(&'a [RecordLiteralField]),
    Field(&'a RecordLiteralField),
    Body(&'a ReferenceBody),
    Bodies(&'a [ReferenceBody]),
    Leaf(Leaf<'a>),
    BeginObject,
    EndObject,
    BeginArray,
    EndArray,
    Name(&'static str),
    Text(&'a str),
    Bool(bool),
    /// An integer member of the eight, spelled as its decimal string.
    Decimal(i64),
    /// A count that stays a JSON number (`maximum_items`).
    Count(u32),
}

/// A member of bounded depth, written through its `FixedShape` serde encoding.
enum Leaf<'a> {
    Integer(&'a IntegerType),
    Rational(&'a RationalType),
    Numeric(NumericOperator),
    Comparison(ComparisonOperator),
    Boolean(BooleanOperator),
    Quantifier(QuantifierKind),
    Domain(QuantifierDomain),
    Observation(StateObservation),
    Dependency(&'a DependencyIdentity),
}

impl Leaf<'_> {
    fn write<S: Sink + ?Sized>(self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        match self {
            Self::Integer(value) => writer.serialize(value),
            Self::Rational(value) => writer.serialize(value),
            Self::Numeric(value) => writer.serialize(&value),
            Self::Comparison(value) => writer.serialize(&value),
            Self::Boolean(value) => writer.serialize(&value),
            Self::Quantifier(value) => writer.serialize(&value),
            Self::Domain(value) => writer.serialize(&value),
            Self::Observation(value) => writer.serialize(&value),
            Self::Dependency(value) => writer.serialize(value),
        }
    }
}

fn walk<S: Sink + ?Sized>(writer: &mut Writer<'_, S>, root: Step<'_>) -> Result<(), Error> {
    let mut stack = vec![root];
    while let Some(step) = stack.pop() {
        match step {
            Step::BeginObject => writer.begin_object()?,
            Step::EndObject => writer.end_object()?,
            Step::BeginArray => writer.begin_array()?,
            Step::EndArray => writer.end_array()?,
            Step::Name(name) => writer.name(name)?,
            Step::Text(text) => writer.string(text)?,
            Step::Bool(value) => writer.bool(value)?,
            Step::Decimal(value) => writer.string(&value.to_string())?,
            Step::Count(value) => writer.integer(i128::from(value))?,
            Step::Leaf(leaf) => leaf.write(writer)?,
            Step::Type(value) => plan_type(&mut stack, value),
            Step::Collection(value) => plan_collection(&mut stack, value),
            Step::Expression(value) => {
                object(&mut stack, [("kind", Step::Node(value.kind()))]);
            }
            Step::Node(kind) => plan_node(&mut stack, kind),
            Step::Items(items) => array(&mut stack, items.iter().map(Step::Expression)),
            Step::Fields(fields) => {
                let mut fields = fields.iter().collect::<Vec<_>>();
                fields.sort_by(|left, right| {
                    unicode_cmp(left.name().as_str(), right.name().as_str())
                });
                array(&mut stack, fields.into_iter().map(Step::Field));
            }
            Step::Field(field) => object(
                &mut stack,
                [
                    ("name", Step::Text(field.name().as_str())),
                    ("value", Step::Expression(field.value())),
                ],
            ),
            Step::Body(body) => plan_body(&mut stack, body),
            Step::Bodies(bodies) => array(&mut stack, bodies.iter().map(Step::Body)),
        }
    }
    Ok(())
}

/// Pushes an object so that its steps pop in the order given: the members'
/// order does not matter, the writer sorts them.
fn object<'a, const N: usize>(stack: &mut Vec<Step<'a>>, members: [(&'static str, Step<'a>); N]) {
    stack.push(Step::EndObject);
    for (name, value) in members.into_iter().rev() {
        stack.push(value);
        stack.push(Step::Name(name));
    }
    stack.push(Step::BeginObject);
}

fn array<'a>(stack: &mut Vec<Step<'a>>, items: impl DoubleEndedIterator<Item = Step<'a>>) {
    stack.push(Step::EndArray);
    stack.extend(items.rev());
    stack.push(Step::BeginArray);
}

fn plan_type<'a>(stack: &mut Vec<Step<'a>>, value: &'a ValueType) {
    match value {
        ValueType::Boolean => object(stack, [("kind", Step::Text("boolean"))]),
        ValueType::Integer { value } => object(
            stack,
            [
                ("kind", Step::Text("integer")),
                ("value", Step::Leaf(Leaf::Integer(value))),
            ],
        ),
        ValueType::Rational { value } => object(
            stack,
            [
                ("kind", Step::Text("rational")),
                ("value", Step::Leaf(Leaf::Rational(value))),
            ],
        ),
        ValueType::Text => object(stack, [("kind", Step::Text("text"))]),
        ValueType::Enum { name } => object(
            stack,
            [
                ("kind", Step::Text("enum")),
                ("name", Step::Text(name.as_str())),
            ],
        ),
        ValueType::Record { name } => object(
            stack,
            [
                ("kind", Step::Text("record")),
                ("name", Step::Text(name.as_str())),
            ],
        ),
        ValueType::Option { value } => object(
            stack,
            [("kind", Step::Text("option")), ("value", Step::Type(value))],
        ),
        ValueType::Collection { value } => object(
            stack,
            [
                ("kind", Step::Text("collection")),
                ("value", Step::Collection(value)),
            ],
        ),
    }
}

fn plan_collection<'a>(stack: &mut Vec<Step<'a>>, value: &'a CollectionType) {
    object(
        stack,
        [
            ("element", Step::Type(value.element())),
            ("maximum_items", Step::Count(value.maximum_items())),
        ],
    );
}

fn plan_body<'a>(stack: &mut Vec<Step<'a>>, body: &'a ReferenceBody) {
    match body {
        ReferenceBody::Literal => object(stack, [("node", Step::Text("literal"))]),
        ReferenceBody::Reference { identity } => object(
            stack,
            [
                ("node", Step::Text("reference")),
                ("identity", Step::Leaf(Leaf::Dependency(identity))),
            ],
        ),
        ReferenceBody::Composite { children } => object(
            stack,
            [
                ("node", Step::Text("composite")),
                ("children", Step::Bodies(children)),
            ],
        ),
    }
}

/// An expression node without its source span (and a quantifier without its
/// local's source span).
fn plan_node<'a>(stack: &mut Vec<Step<'a>>, kind: &'a ExpressionKind) {
    let expression = |value: &'a Expression| Step::Expression(value);
    match kind {
        ExpressionKind::BooleanLiteral { value } => object(
            stack,
            [
                ("node", Step::Text("boolean_literal")),
                ("value", Step::Bool(*value)),
            ],
        ),
        ExpressionKind::IntegerLiteral { value, value_type } => object(
            stack,
            [
                ("node", Step::Text("integer_literal")),
                ("value", Step::Decimal(*value)),
                ("value_type", Step::Leaf(Leaf::Integer(value_type))),
            ],
        ),
        ExpressionKind::RationalLiteral {
            numerator,
            denominator,
            value_type,
        } => object(
            stack,
            [
                ("node", Step::Text("rational_literal")),
                ("numerator", Step::Decimal(*numerator)),
                ("denominator", Step::Decimal(*denominator)),
                ("value_type", Step::Leaf(Leaf::Rational(value_type))),
            ],
        ),
        ExpressionKind::TextLiteral { value } => object(
            stack,
            [
                ("node", Step::Text("text_literal")),
                ("value", Step::Text(value)),
            ],
        ),
        ExpressionKind::EnumLiteral {
            enumeration,
            variant,
        } => object(
            stack,
            [
                ("node", Step::Text("enum_literal")),
                ("enumeration", Step::Text(enumeration.as_str())),
                ("variant", Step::Text(variant.as_str())),
            ],
        ),
        ExpressionKind::OptionNone { value_type } => object(
            stack,
            [
                ("node", Step::Text("option_none")),
                ("value_type", Step::Type(value_type)),
            ],
        ),
        ExpressionKind::OptionSome { value_type, value } => object(
            stack,
            [
                ("node", Step::Text("option_some")),
                ("value_type", Step::Type(value_type)),
                ("value", expression(value)),
            ],
        ),
        ExpressionKind::RecordLiteral { record, fields } => object(
            stack,
            [
                ("node", Step::Text("record_literal")),
                ("record", Step::Text(record.as_str())),
                ("fields", Step::Fields(fields)),
            ],
        ),
        ExpressionKind::CollectionLiteral { value_type, items } => object(
            stack,
            [
                ("node", Step::Text("collection_literal")),
                ("value_type", Step::Collection(value_type)),
                ("items", Step::Items(items)),
            ],
        ),
        ExpressionKind::ValueReference { name, observation } => object(
            stack,
            [
                ("node", Step::Text("value_reference")),
                ("name", Step::Text(name.as_str())),
                ("observation", Step::Leaf(Leaf::Observation(*observation))),
            ],
        ),
        ExpressionKind::LocalReference { name } => object(
            stack,
            [
                ("node", Step::Text("local_reference")),
                ("name", Step::Text(name.as_str())),
            ],
        ),
        ExpressionKind::FieldAccess { base, field } => object(
            stack,
            [
                ("node", Step::Text("field_access")),
                ("base", expression(base)),
                ("field", Step::Text(field.as_str())),
            ],
        ),
        ExpressionKind::IsPresent { option } => object(
            stack,
            [
                ("node", Step::Text("is_present")),
                ("option", expression(option)),
            ],
        ),
        ExpressionKind::Unwrap { option } => object(
            stack,
            [
                ("node", Step::Text("unwrap")),
                ("option", expression(option)),
            ],
        ),
        ExpressionKind::Length { collection } => object(
            stack,
            [
                ("node", Step::Text("length")),
                ("collection", expression(collection)),
            ],
        ),
        ExpressionKind::Index { collection, index } => object(
            stack,
            [
                ("node", Step::Text("index")),
                ("collection", expression(collection)),
                ("index", expression(index)),
            ],
        ),
        ExpressionKind::Call {
            function,
            arguments,
        } => object(
            stack,
            [
                ("node", Step::Text("call")),
                ("function", Step::Text(function.as_str())),
                ("arguments", Step::Items(arguments)),
            ],
        ),
        ExpressionKind::Numeric {
            operator,
            left,
            right,
        } => object(
            stack,
            [
                ("node", Step::Text("numeric")),
                ("operator", Step::Leaf(Leaf::Numeric(*operator))),
                ("left", expression(left)),
                ("right", expression(right)),
            ],
        ),
        ExpressionKind::NumericNegate { operand } => object(
            stack,
            [
                ("node", Step::Text("numeric_negate")),
                ("operand", expression(operand)),
            ],
        ),
        ExpressionKind::Compare {
            operator,
            left,
            right,
        } => object(
            stack,
            [
                ("node", Step::Text("compare")),
                ("operator", Step::Leaf(Leaf::Comparison(*operator))),
                ("left", expression(left)),
                ("right", expression(right)),
            ],
        ),
        ExpressionKind::BooleanNot { operand } => object(
            stack,
            [
                ("node", Step::Text("boolean_not")),
                ("operand", expression(operand)),
            ],
        ),
        ExpressionKind::Boolean {
            operator,
            left,
            right,
        } => object(
            stack,
            [
                ("node", Step::Text("boolean")),
                ("operator", Step::Leaf(Leaf::Boolean(*operator))),
                ("left", expression(left)),
                ("right", expression(right)),
            ],
        ),
        ExpressionKind::Quantifier {
            quantifier,
            domain,
            collection,
            local,
            local_source: _,
            predicate,
        } => object(
            stack,
            [
                ("node", Step::Text("quantifier")),
                ("quantifier", Step::Leaf(Leaf::Quantifier(*quantifier))),
                ("domain", Step::Leaf(Leaf::Domain(*domain))),
                ("collection", expression(collection)),
                ("local", Step::Text(local.as_str())),
                ("predicate", expression(predicate)),
            ],
        ),
    }
}

#[cfg(test)]
mod tests {
    use quire_canonical::{LimitExceeded, LimitKind, ProtocolViolation};

    use super::*;

    /// Tracing: TC-017, FR-016-AC-6.
    #[ix_trace_rs::trace("TC-017", "FR-016-AC-6")]
    #[test]
    fn tc_017_a_limit_and_an_encoder_fault_share_the_code_but_not_the_message() {
        let limit = Error::Limit(LimitExceeded {
            kind: LimitKind::CanonicalBytes,
            bound: 1,
            required: 2,
        });
        let fault = Error::Protocol(ProtocolViolation::MismatchedEnd);
        let refused = encoder_refusal(&limit, "package", None);
        let faulted = encoder_refusal(&fault, "package", None);
        assert_eq!(
            refused.code,
            DiagnosticCode::CanonicalizationResourceExhausted
        );
        assert_eq!(faulted.code, refused.code);
        assert_eq!(
            refused.message,
            "canonical byte allocation exceeded available resources"
        );
        assert!(
            faulted
                .message
                .starts_with("canonical encoder failed unexpectedly"),
            "{}",
            faulted.message
        );
    }
}
