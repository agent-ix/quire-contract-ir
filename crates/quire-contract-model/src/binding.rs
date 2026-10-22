//! IR-owned decoding of derived executable projections. This boundary validates
//! binding and semantics, not the authenticity or correctness of a frontend.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::OnceLock;

#[cfg(test)]
thread_local! {
    static DECLARATION_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

use quire_canonical::{FixedShape, Limits};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;

use crate::{
    CanonicalDigest, CanonicalProfile, ClauseKind, ClauseRef, ContractPackage,
    DeclarationEnvironment, Diagnostic, DiagnosticCode, ExecutionPoint, ReferenceBody, SourceSpan,
    TypedExpression, ValidationOptions, CONFORMANCE_SCHEMA_ID, MAX_CONFORMANCE_FILE_BYTES,
    MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_NODES, MAX_WIRE_JSON_DEPTH, PACKAGE_SCHEMA_ID,
};

pub const EXECUTABLE_PROJECTION_FORMAT: &str = "quire.contract.executable-projection/v1";
pub const EXECUTABLE_PROJECTION_SCHEMA: &str =
    include_str!("../../../schemas/contract-executable-projection-v1.schema.json");
/// Domain-separated canonical identity profile; not a producer attestation.
pub const BOUND_IDENTITY_PROFILE: &str = "quire.contract.bound-identity/v1";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoundClause {
    identity: ClauseRef,
    source_order: u32,
    kind: ClauseKind,
    anchor: ExecutionPoint,
    source: SourceSpan,
    environment: DeclarationEnvironment,
    expression: TypedExpression,
    declaration_digest: CanonicalDigest,
    expression_digest: CanonicalDigest,
}

impl BoundClause {
    pub fn identity(&self) -> &ClauseRef {
        &self.identity
    }
    /// Zero-based authored order among executable clauses in the source package.
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    pub fn kind(&self) -> ClauseKind {
        self.kind
    }
    pub fn anchor(&self) -> &ExecutionPoint {
        &self.anchor
    }
    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
    pub fn environment(&self) -> &DeclarationEnvironment {
        &self.environment
    }
    pub fn expression(&self) -> &TypedExpression {
        &self.expression
    }
    pub fn declaration_digest(&self) -> CanonicalDigest {
        self.declaration_digest
    }
    pub fn expression_digest(&self) -> CanonicalDigest {
        self.expression_digest
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoundPackage {
    package: ContractPackage<ReferenceBody>,
    clauses: Vec<BoundClause>,
    informational: Vec<ClauseRef>,
    digest: CanonicalDigest,
}

/// The bound identity envelope (`quire.contract.bound-identity/v1`): its depth
/// is fixed by its type, it holds no `serde_json::Value`, and its digests
/// serialize as lowercase hexadecimal.
#[derive(Serialize, FixedShape)]
struct BoundIdentityEnvelope<'a> {
    profile: &'static str,
    canonical_profile: &'static str,
    package: CanonicalDigest,
    bindings: Vec<BoundBindingIdentity<'a>>,
}

/// One executable clause's identity and the digests of its declaration and
/// expression.
#[derive(Serialize, FixedShape)]
struct BoundBindingIdentity<'a> {
    clause: &'a ClauseRef,
    declaration: CanonicalDigest,
    expression: CanonicalDigest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Projection {
    format: String,
    package: Value,
    bindings: Vec<Binding>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    clause: ClauseRef,
    expression: Value,
}

// Preserve strictness before Value can erase duplicate member occurrences.
// This visitor owns no wire semantics; the published schemas and existing
// typed decoders remain the authorities for field names and value types.
struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON with unique object members")
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<StrictValue, E> {
                Ok(StrictValue(Value::Bool(value)))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<StrictValue, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<StrictValue, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<StrictValue, E> {
                serde_json::Number::from_f64(value)
                    .map(|number| StrictValue(Value::Number(number)))
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<StrictValue, E> {
                Ok(StrictValue(Value::String(value.to_owned())))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<StrictValue, E> {
                Ok(StrictValue(Value::String(value)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<StrictValue, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<StrictValue, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictValue(value)) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<StrictValue, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = object.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("duplicate JSON member"));
                    }
                    let StrictValue(value) = object.next_value()?;
                    values.insert(key, value);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        decoder.deserialize_any(StrictVisitor)
    }
}

fn failure(code: DiagnosticCode, message: &str, path: &str) -> Vec<Diagnostic> {
    vec![Diagnostic::error(code, message, path)]
}

fn invalid(message: &str) -> Vec<Diagnostic> {
    failure(DiagnosticCode::InvalidWireFormat, message, "projection")
}

fn budget(nodes: &mut u32, additional: u32) -> Result<(), Vec<Diagnostic>> {
    *nodes = nodes.saturating_add(additional);
    if *nodes > MAX_SEMANTIC_NODES {
        return Err(failure(
            DiagnosticCode::SemanticInputTooLarge,
            "aggregate projection semantic node budget exceeded",
            "projection",
        ));
    }
    Ok(())
}

impl BoundPackage {
    pub fn package(&self) -> &ContractPackage<ReferenceBody> {
        &self.package
    }
    pub fn clauses(&self) -> &[BoundClause] {
        &self.clauses
    }
    pub fn informational(&self) -> &[ClauseRef] {
        &self.informational
    }
    pub fn digest(&self) -> CanonicalDigest {
        self.digest
    }

    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, Vec<Diagnostic>> {
        if bytes.len() as u64 > MAX_CONFORMANCE_FILE_BYTES
            || crate::limits::json_nesting_exceeds(bytes, MAX_WIRE_JSON_DEPTH)
        {
            return Err(failure(
                DiagnosticCode::SemanticInputTooLarge,
                "projection byte or wire depth limit exceeded",
                "projection",
            ));
        }
        // The syntactic ceiling is checked before reserving recursive-work
        // space. Schema checking and typed construction also recurse, so
        // protecting just Serde is insufficient on small caller stacks.
        // Stay on the calling thread; no thread-creation capability is needed.
        stacker::maybe_grow(16 * 1024 * 1024, 16 * 1024 * 1024, || {
            Self::decode_bounded(bytes)
        })
    }

    fn decode_bounded(bytes: &[u8]) -> Result<Self, Vec<Diagnostic>> {
        let mut decoder = serde_json::Deserializer::from_slice(bytes);
        decoder.disable_recursion_limit();
        let StrictValue(value) = StrictValue::deserialize(serde_stacker::Deserializer::new(
            &mut decoder,
        ))
        .map_err(|_| invalid("projection must be valid UTF-8 JSON with unique object members"))?;
        decoder
            .end()
            .map_err(|_| invalid("trailing JSON is not allowed"))?;
        // Check collections before schema evaluation or further deserialization.
        let mut pending = vec![&value];
        while let Some(value) = pending.pop() {
            match value {
                Value::Array(items) => {
                    if items.len() > MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                        return Err(failure(
                            DiagnosticCode::SemanticInputTooLarge,
                            "projection collection limit exceeded",
                            "projection",
                        ));
                    }
                    pending.extend(items);
                }
                Value::Object(fields) => pending.extend(fields.values()),
                _ => {}
            }
        }
        validate_schema(&value)?;
        let projection: Projection =
            serde_json::from_value(value).map_err(|_| invalid("invalid projection envelope"))?;
        if projection.format != EXECUTABLE_PROJECTION_FORMAT {
            return Err(invalid("unsupported executable projection format"));
        }
        let package =
            ContractPackage::from_json_value(&projection.package, ValidationOptions::strict())?;
        let mut nodes = 0;
        let mut expected = BTreeMap::new();
        let mut clauses_by_ref = BTreeMap::new();
        let mut informational = Vec::new();
        for requirement in package.requirements() {
            budget(&mut nodes, 1)?;
            for clause in requirement.clauses() {
                #[cfg(test)]
                DECLARATION_VISITS.with(|visits| visits.set(visits.get() + 1));
                budget(&mut nodes, 1)?;
                let mut bodies = vec![clause.body()];
                while let Some(body) = bodies.pop() {
                    budget(&mut nodes, 1)?;
                    if let ReferenceBody::Composite { children } = body {
                        bodies.extend(children);
                    }
                }
                let identity = package.clause_ref(requirement, clause);
                clauses_by_ref.insert(identity.clone(), clause);
                if clause.kind().executable() {
                    expected.insert(identity, clause);
                } else {
                    informational.push(identity);
                }
            }
        }
        let mut authored: Vec<_> = expected.iter().collect();
        authored.sort_by(|(left_identity, left), (right_identity, right)| {
            left.source()
                .cmp(right.source())
                .then_with(|| left_identity.cmp(right_identity))
        });
        let mut source_order = BTreeMap::new();
        for (index, (identity, _)) in authored.into_iter().enumerate() {
            let order = u32::try_from(index).map_err(|_| {
                failure(
                    DiagnosticCode::SemanticInputTooLarge,
                    "executable clause order exceeds the supported range",
                    "projection.package.requirements.clauses",
                )
            })?;
            source_order.insert(identity.clone(), order);
        }
        informational.sort();
        let mut bound = BTreeMap::new();
        for binding in projection.bindings {
            #[cfg(test)]
            DECLARATION_VISITS.with(|visits| visits.set(visits.get() + 1));
            let clause = match clauses_by_ref.get(&binding.clause).copied() {
                Some(clause) => clause,
                None => package
                    .resolve_clause(&binding.clause, None)
                    .map_err(|diagnostic| vec![diagnostic])?,
            };
            if !clause.kind().executable() {
                return Err(failure(
                    DiagnosticCode::InformationalClauseAnchored,
                    "informational clause cannot have an executable binding",
                    "bindings.clause",
                ));
            }
            if bound.contains_key(&binding.clause) {
                return Err(failure(
                    DiagnosticCode::DuplicateClause,
                    "duplicate executable binding",
                    "bindings.clause",
                ));
            }
            let anchor = clause.anchor().ok_or_else(|| {
                failure(
                    DiagnosticCode::FloatingExecutableClause,
                    "missing executable anchor",
                    "bindings.clause",
                )
            })?;
            let checked = crate::wire::check_expression_input(
                binding.expression,
                Some((binding.clause.requirement(), anchor)),
            )?;
            budget(&mut nodes, checked.semantic_nodes)?;
            if checked.expression.dependencies() != &clause.dependencies() {
                return Err(failure(
                    DiagnosticCode::MalformedReference,
                    "expression dependencies disagree with clause metadata",
                    "bindings.expression",
                ));
            }
            let declaration_digest = checked
                .environment
                .canonical_declaration(CanonicalProfile::V1)
                .map_err(|diagnostic| vec![diagnostic])?
                .digest();
            let expression_digest = checked
                .expression
                .canonical_expression(CanonicalProfile::V1)
                .map_err(|diagnostic| vec![diagnostic])?
                .digest();
            bound.insert(
                binding.clause.clone(),
                BoundClause {
                    source_order: *source_order.get(&binding.clause).ok_or_else(|| {
                        failure(
                            DiagnosticCode::OrphanedClauseReference,
                            "executable binding has no authored source order",
                            "bindings.clause",
                        )
                    })?,
                    identity: binding.clause,
                    kind: clause.kind(),
                    anchor: anchor.clone(),
                    source: clause.source().clone(),
                    environment: checked.environment,
                    expression: checked.expression,
                    declaration_digest,
                    expression_digest,
                },
            );
        }
        if expected
            .keys()
            .any(|identity| !bound.contains_key(identity))
        {
            return Err(failure(
                DiagnosticCode::OrphanedClauseReference,
                "an executable clause has no binding",
                "bindings",
            ));
        }
        let clauses: Vec<_> = bound.into_values().collect();
        let package_digest = package
            .canonical_package(CanonicalProfile::V1)
            .map_err(|diagnostic| vec![diagnostic])?
            .digest();
        let envelope = BoundIdentityEnvelope {
            profile: BOUND_IDENTITY_PROFILE,
            canonical_profile: CanonicalProfile::V1.as_str(),
            package: package_digest,
            bindings: clauses
                .iter()
                .map(|clause| BoundBindingIdentity {
                    clause: clause.identity(),
                    declaration: clause.declaration_digest(),
                    expression: clause.expression_digest(),
                })
                .collect(),
        };
        // SHA-256 of the envelope's canonical bytes, with no domain prefix:
        // the profile member is the separation.
        let digest = quire_canonical::sha256(&envelope, Limits::new(MAX_CONFORMANCE_FILE_BYTES))
            .map_err(|error| {
                vec![crate::canonical::encoder_refusal(
                    &error,
                    "projection.identity",
                    None,
                )]
            })
            .and_then(|digest| {
                CanonicalDigest::parse(&digest.to_string()).map_err(|diagnostic| vec![diagnostic])
            })?;
        Ok(Self {
            package,
            clauses,
            informational,
            digest,
        })
    }
}

static COMPILED_PROJECTION_SCHEMA: OnceLock<Result<jsonschema::JSONSchema, &'static str>> =
    OnceLock::new();

fn compiled_projection_schema() -> Result<&'static jsonschema::JSONSchema, Vec<Diagnostic>> {
    COMPILED_PROJECTION_SCHEMA
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(EXECUTABLE_PROJECTION_SCHEMA)
                .map_err(|_| "invalid embedded projection schema")?;
            let package: Value = serde_json::from_str(include_str!(
                "../../../schemas/contract-package-reference-v1.schema.json"
            ))
            .map_err(|_| "invalid embedded package schema")?;
            let conformance: Value = serde_json::from_str(include_str!(
                "../../../schemas/contract-conformance-fixture-v1.schema.json"
            ))
            .map_err(|_| "invalid embedded expression schema")?;
            jsonschema::JSONSchema::options()
                .with_draft(jsonschema::Draft::Draft7)
                .with_document(PACKAGE_SCHEMA_ID.to_owned(), package)
                .with_document(CONFORMANCE_SCHEMA_ID.to_owned(), conformance)
                .compile(&schema)
                .map_err(|_| "cannot compile embedded projection schema")
        })
        .as_ref()
        .map_err(|message| invalid(message))
}

fn validate_schema(value: &Value) -> Result<(), Vec<Diagnostic>> {
    let compiled = compiled_projection_schema()?;
    if !compiled.is_valid(value) {
        return Err(invalid("projection does not satisfy its normative schema"));
    }
    Ok(())
}

#[cfg(test)]
mod schema_cache_tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn fixture() -> Value {
        let mut package: Value = serde_json::from_str(include_str!(
            "../../../corpus/contract-v0.1/inputs/package-constructs.json"
        ))
        .expect("package fixture");
        let mut bindings = Vec::new();
        for requirement in package["requirements"]
            .as_array_mut()
            .expect("requirements")
        {
            let owner = json!({"package":"agent-ix/conformance",
                "requirement":requirement["id"], "revision":requirement["revision"]});
            for clause in requirement["clauses"].as_array_mut().expect("clauses") {
                clause["body"] = json!({"node":"literal"});
                if clause["kind"] == "information" {
                    continue;
                }
                let mut expression: Value = serde_json::from_str(include_str!(
                    "../../../corpus/contract-v0.1/inputs/expression-boolean-literal.json"
                ))
                .expect("expression fixture");
                expression["owner"] = owner.clone();
                expression["execution_point"] = clause["anchor"].clone();
                expression["clause_root"] = json!(true);
                bindings.push(json!({"clause":{"requirement":owner,"clause":clause["id"]},
                    "expression":expression}));
            }
        }
        json!({"format":EXECUTABLE_PROJECTION_FORMAT,"package":package,"bindings":bindings})
    }

    fn decode(value: &Value) -> Result<BoundPackage, Vec<Diagnostic>> {
        BoundPackage::from_json_bytes(&serde_json::to_vec(value).expect("projection bytes"))
    }

    #[trace("TC-035", "FR-023-AC-7")]
    #[test]
    fn projection_schema_and_declaration_index_are_reused_across_decodes() {
        let valid = fixture();
        let first_bound = decode(&valid).expect("valid projection");
        let first_visits = DECLARATION_VISITS.with(std::cell::Cell::get);
        let declarations = first_bound.clauses().len() + first_bound.informational().len();
        let bindings = valid["bindings"].as_array().expect("bindings").len();
        assert_eq!(declarations, 6);
        assert_eq!(bindings, 5);
        assert_eq!(first_visits, declarations + bindings);
        let first = compiled_projection_schema().expect("embedded schema");
        let second_bound = decode(&valid).expect("same valid projection");
        assert_eq!(first_bound.digest(), second_bound.digest());
        assert_eq!(
            DECLARATION_VISITS.with(std::cell::Cell::get) - first_visits,
            declarations + bindings
        );
        let mut invalid_clause = valid.clone();
        invalid_clause["bindings"][0]["clause"]["clause"] = json!("missing");
        let first_error = decode(&invalid_clause).expect_err("missing clause");
        let second_error = decode(&invalid_clause).expect_err("same missing clause");
        assert_eq!(first_error, second_error);
        let mut invalid_schema = valid;
        invalid_schema["format"] = json!("future");
        let first_schema_error = decode(&invalid_schema).expect_err("invalid schema");
        let second_schema_error = decode(&invalid_schema).expect_err("same invalid schema");
        assert_eq!(first_schema_error, second_schema_error);
        let second = compiled_projection_schema().expect("same embedded schema");
        assert!(std::ptr::eq(first, second));
    }
}
