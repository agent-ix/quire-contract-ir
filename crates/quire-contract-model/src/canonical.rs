use std::{cmp::Ordering, fmt};

use quire_canonical::Limits;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Number, Value};
use sha2::{Digest as _, Sha256};

use crate::{
    Clause, ContractPackage, DeclarationEnvironment, DependencySource, Diagnostic, DiagnosticCode,
    ReferenceBody, Requirement, RequirementRef, SchemaVersion, SourceSpan, TypedExpression,
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

pub trait CanonicalBody: Clone + DependencySource + Eq + Serialize {
    fn canonical_body_value(&self) -> Result<Value, Diagnostic>;
}

impl CanonicalBody for ReferenceBody {
    fn canonical_body_value(&self) -> Result<Value, Diagnostic> {
        semantic_value(self, "clause.body")
    }
}

impl CanonicalBody for TypedExpression {
    fn canonical_body_value(&self) -> Result<Value, Diagnostic> {
        typed_expression_value(self)
    }
}

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
            package_value(self)?,
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
            requirement_value(self, requirement)?,
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
            clause_value(self.requirement_ref(requirement), clause)?,
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
            declaration_value(self)?,
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
            typed_expression_value(self)?,
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

fn package_value<B: CanonicalBody>(package: &ContractPackage<B>) -> Result<Value, Diagnostic> {
    let mut requirements = package.requirements().iter().collect::<Vec<_>>();
    requirements.sort_by(|left, right| unicode_cmp(left.id().as_str(), right.id().as_str()));
    let requirements = requirements
        .into_iter()
        .map(|requirement| requirement_value(package, requirement))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(object([
        ("id", Value::String(package.id().as_str().to_owned())),
        ("requirements", Value::Array(requirements)),
        (
            "schema_version",
            schema_version_value(package.schema_version()),
        ),
    ]))
}

fn requirement_value<B: CanonicalBody>(
    package: &ContractPackage<B>,
    requirement: &Requirement<B>,
) -> Result<Value, Diagnostic> {
    let reference = package.requirement_ref(requirement);
    let mut clauses = requirement.clauses().iter().collect::<Vec<_>>();
    clauses.sort_by(|left, right| unicode_cmp(left.id().as_str(), right.id().as_str()));
    let clauses = clauses
        .into_iter()
        .map(|clause| clause_value(reference.clone(), clause))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(object([
        ("clauses", Value::Array(clauses)),
        ("id", Value::String(requirement.id().as_str().to_owned())),
        ("package", Value::String(package.id().as_str().to_owned())),
        (
            "revision",
            Value::Number(Number::from(requirement.revision().get())),
        ),
    ]))
}

fn clause_value<B: CanonicalBody>(
    requirement: RequirementRef,
    clause: &Clause<B>,
) -> Result<Value, Diagnostic> {
    let mut body = clause.body().canonical_body_value()?;
    normalize_semantic_sets(&mut body);
    let mut members = Map::new();
    if let Some(anchor) = clause.anchor() {
        members.insert(
            "anchor".to_owned(),
            semantic_value(anchor, "clause.anchor")?,
        );
    }
    members.insert("body".to_owned(), body);
    members.insert(
        "id".to_owned(),
        Value::String(clause.id().as_str().to_owned()),
    );
    members.insert(
        "kind".to_owned(),
        semantic_value(&clause.kind(), "clause.kind")?,
    );
    members.insert(
        "requirement".to_owned(),
        semantic_value(&requirement, "clause.requirement")?,
    );
    Ok(Value::Object(members))
}

fn declaration_value(environment: &DeclarationEnvironment) -> Result<Value, Diagnostic> {
    let mut types = environment
        .types()
        .iter()
        .map(|value| semantic_value(value, "declaration.types"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut values = environment
        .values()
        .iter()
        .map(|value| semantic_value(value, "declaration.values"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut functions = environment
        .functions()
        .iter()
        .map(|value| semantic_value(value, "declaration.functions"))
        .collect::<Result<Vec<_>, _>>()?;
    for value in types
        .iter_mut()
        .chain(values.iter_mut())
        .chain(functions.iter_mut())
    {
        strip_source_metadata(value);
        normalize_semantic_sets(value);
    }
    types.sort_by(|left, right| unicode_cmp(nested_name(left), nested_name(right)));
    values.sort_by(|left, right| unicode_cmp(direct_name(left), direct_name(right)));
    functions.sort_by(|left, right| unicode_cmp(direct_name(left), direct_name(right)));
    Ok(object([
        ("functions", Value::Array(functions)),
        (
            "owner",
            semantic_value(environment.owner(), "declaration.owner")?,
        ),
        ("types", Value::Array(types)),
        ("values", Value::Array(values)),
    ]))
}

fn typed_expression_value(expression: &TypedExpression) -> Result<Value, Diagnostic> {
    let mut tree = semantic_value(expression.expression(), "expression.tree")?;
    strip_source_metadata(&mut tree);
    normalize_semantic_sets(&mut tree);
    Ok(object([
        (
            "result_type",
            semantic_value(expression.value_type(), "expression.result_type")?,
        ),
        ("tree", tree),
    ]))
}

fn schema_version_value(version: SchemaVersion) -> Value {
    object([
        ("major", Value::Number(Number::from(version.major()))),
        ("minor", Value::Number(Number::from(version.minor()))),
    ])
}

fn semantic_value<T: Serialize>(value: &T, path: &str) -> Result<Value, Diagnostic> {
    serde_json::to_value(value).map_err(|error| {
        Diagnostic::error(
            DiagnosticCode::CanonicalizationResourceExhausted,
            format!("semantic projection failed: {error}"),
            path,
        )
    })
}

fn object<const N: usize>(members: [(&str, Value); N]) -> Value {
    Value::Object(
        members
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

fn strip_source_metadata(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                strip_source_metadata(value);
            }
        }
        Value::Object(members) => {
            members.remove("source");
            members.remove("local_source");
            members.remove("proof_span");
            members.remove("nodes");
            members.remove("obligations");
            members.remove("dependencies");
            for value in members.values_mut() {
                strip_source_metadata(value);
            }
        }
        _ => {}
    }
}

fn normalize_semantic_sets(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                normalize_semantic_sets(value);
            }
        }
        Value::Object(members) => {
            for value in members.values_mut() {
                normalize_semantic_sets(value);
            }
            if members.get("node").and_then(Value::as_str) == Some("record_literal") {
                sort_named_array(members.get_mut("fields"));
            }
            match members.get("kind").and_then(Value::as_str) {
                Some("enum") => sort_nested_named_array(members, "variants"),
                Some("record") => sort_nested_named_array(members, "fields"),
                _ => {}
            }
        }
        _ => {}
    }
}

fn sort_nested_named_array(members: &mut Map<String, Value>, key: &str) {
    let Some(Value::Object(declaration)) = members.get_mut("declaration") else {
        return;
    };
    sort_named_array(declaration.get_mut(key));
}

fn sort_named_array(value: Option<&mut Value>) {
    if let Some(Value::Array(values)) = value {
        values.sort_by(|left, right| unicode_cmp(direct_name(left), direct_name(right)));
    }
}

fn direct_name(value: &Value) -> &str {
    value.get("name").and_then(Value::as_str).unwrap_or("")
}

fn nested_name(value: &Value) -> &str {
    value
        .get("declaration")
        .and_then(|declaration| declaration.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn unicode_cmp(left: &str, right: &str) -> Ordering {
    left.chars().cmp(right.chars())
}

fn canonicalize(
    kind: CanonicalKind,
    value: Value,
    maximum_bytes: u64,
    path: &str,
    span: Option<&SourceSpan>,
) -> Result<CanonicalOutput, Diagnostic> {
    let mut envelope = object([
        ("kind", Value::String(kind.as_str().to_owned())),
        ("profile", Value::String(CANONICAL_PROFILE.to_owned())),
        ("value", value),
    ]);
    project_profile_values(&mut envelope, path, span)?;
    let bytes = CanonicalBytes(
        quire_canonical::to_vec(&envelope, encoder_limits(maximum_bytes))
            .map_err(|error| refusal(&error, path, span))?,
    );
    let digest = digest(kind, bytes.as_slice());
    Ok(CanonicalOutput {
        kind,
        bytes,
        digest,
    })
}

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

/// The `quire-canonical` limits for one encode: the caller's byte budget and
/// the deepest nesting that encoder accepts. Every identity step in this crate
/// takes its limits from here, so the depth bound is named once.
pub(crate) const fn encoder_limits(maximum_bytes: u64) -> Limits {
    match Limits::new(maximum_bytes, Limits::MAX_DEPTH) {
        Ok(limits) => limits,
        Err(_) => panic!("MAX_DEPTH is within MAX_DEPTH"),
    }
}

/// The largest integer magnitude `quire-canonical` encodes as a JSON number:
/// 2^53, the largest integer every smaller one, and it, holds exactly as an
/// IEEE 754 double (RFC 8785 spells every number as one).
const LARGEST_EXACT_INTEGER: u64 = 1 << 53;

/// Brings `value` to the `quire.contract.canonical-json/v1` profile's
/// spelling before it is encoded: an integer whose magnitude exceeds 2^53 has
/// no exact RFC 8785 number, so it is spelled as its decimal string (the
/// bound of a numeric type, for instance). Refuses the two kinds of value the
/// profile has no spelling for and `quire-canonical` would otherwise encode:
/// `null` and a non-integer number. Iterative, so the walk's own depth is not
/// bounded by the stack.
fn project_profile_values(
    value: &mut Value,
    path: &str,
    span: Option<&SourceSpan>,
) -> Result<(), Diagnostic> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Null => {
                return Err(invalid_content(
                    "null is not canonical semantic content",
                    path,
                    span,
                ))
            }
            Value::Number(number) => {
                let magnitude = match (number.as_u64(), number.as_i64()) {
                    (Some(unsigned), _) => unsigned,
                    (None, Some(signed)) => signed.unsigned_abs(),
                    (None, None) => {
                        return Err(invalid_content(
                            "floating-point values are not canonical",
                            path,
                            span,
                        ))
                    }
                };
                if magnitude > LARGEST_EXACT_INTEGER {
                    *value = Value::String(number.to_string());
                }
            }
            Value::Bool(_) | Value::String(_) => {}
            Value::Array(values) => pending.extend(values.iter_mut()),
            Value::Object(members) => pending.extend(members.values_mut()),
        }
    }
    Ok(())
}

fn invalid_content(message: &str, path: &str, span: Option<&SourceSpan>) -> Diagnostic {
    let diagnostic = Diagnostic::error(DiagnosticCode::InvalidWireFormat, message, path);
    match span {
        Some(span) => diagnostic.at_span(span),
        None => diagnostic,
    }
}

/// The diagnostic for an encode `quire-canonical` refused: a reached limit or a
/// failed reservation is `canonicalization_resource_exhausted`; a value it has
/// no RFC 8785 spelling for (an integer past 2^53, a non-finite number) is
/// `invalid_wire_format`.
pub(crate) fn refusal(
    error: &quire_canonical::Error,
    path: &str,
    span: Option<&SourceSpan>,
) -> Diagnostic {
    let diagnostic = match error {
        quire_canonical::Error::Limit(_) | quire_canonical::Error::Allocation { .. } => {
            Diagnostic::error(
                DiagnosticCode::CanonicalizationResourceExhausted,
                "canonical byte allocation exceeded available resources",
                path,
            )
        }
        other => Diagnostic::error(DiagnosticCode::InvalidWireFormat, other.to_string(), path),
    };
    match span {
        Some(span) => diagnostic.at_span(span),
        None => diagnostic,
    }
}
